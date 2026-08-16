use crate::{AppState, driver::driver_events::DriverEvent, error::DriverEventError};
use axum::Json;
use axum::extract::{Path, State};
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use florca_core::run::RunId;
use std::sync::Arc;
use tracing::{debug, error};

pub async fn handle_events_batch(
    Path(run_id): Path<RunId>,
    State(state): State<Arc<AppState>>,
    Json(events): Json<Vec<DriverEvent>>,
) -> axum::response::Result<(), DriverEventError> {
    debug!(run = %run_id, "Received event batch with {} events", events.len());

    let run_exists = state
        .run_service
        .run_exists(run_id)
        .await
        .map_err(DriverEventError::Other)?;
    if !run_exists {
        return Err(DriverEventError::NotFound(run_id));
    }

    if events.is_empty() {
        return Ok(());
    }

    let processor = state.run_service.new_event_processor(run_id);
    processor
        .process_events_batch(events)
        .await
        .map_err(DriverEventError::Other)?;

    Ok(())
}

impl IntoResponse for DriverEventError {
    fn into_response(self) -> Response {
        match self {
            DriverEventError::NotFound(_) => {
                (StatusCode::NOT_FOUND, self.to_string()).into_response()
            }
            DriverEventError::Other(err) => {
                error!("{:?}", err);
                (StatusCode::INTERNAL_SERVER_ERROR, format!("{err:#}")).into_response()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::deployer_client::DeployerClientImpl;
    use crate::driver::driver_events::{
        InvocationEvent, InvocationFailureEvent, InvocationSuccessEvent, LogEvent, LogLevel,
        WorkflowLogMessage,
    };
    use crate::inspection::InspectionService;
    use crate::kill::KillService;
    use crate::message::MessageService;
    use crate::process::ProcessManager;
    use crate::ps::PsService;
    use crate::repository::{EngineRepository, SqlxEngineRepository};
    use crate::run::RunService;
    use anyhow::Result;
    use chrono::Utc;
    use florca_core::invocation::InvocationId;
    use florca_core::run::RunRequest;
    use serde_json::json;
    use testcontainers_modules::{postgres, testcontainers::runners::AsyncRunner};

    fn new_app_state(repository: Arc<SqlxEngineRepository>) -> Arc<AppState> {
        let process_manager = Arc::new(ProcessManager::new());
        Arc::new(AppState {
            ps_service: Arc::new(PsService::new(repository.clone(), process_manager.clone())),
            message_service: Arc::new(MessageService::new(process_manager.clone())),
            kill_service: Arc::new(KillService::new(process_manager.clone(), repository.clone())),
            inspection_service: Arc::new(InspectionService::new(
                repository.clone(),
                process_manager.clone(),
            )),
            run_service: Arc::new(RunService::new(
                process_manager,
                repository,
                Arc::new(DeployerClientImpl),
            )),
        })
    }

    #[tokio::test]
    async fn test_event_batch_persistence() -> Result<()> {
        // Initialize the test environment

        let container = postgres::Postgres::default().start().await?;
        let host = container.get_host().await?;
        let host_port = container.get_host_port_ipv4(5432).await?;
        let database_url = format!("postgres://postgres:postgres@{host}:{host_port}/postgres");
        let repository =
            Arc::new(SqlxEngineRepository::setup_with_database_url(&database_url).await?);
        let state = new_app_state(repository.clone());

        let run_request = RunRequest {
            deployment_name: "test-deployment".into(),
            entry_point: "start".into(),
            input: json!({}),
            params: json!(null),
        };
        let run = repository.new_run(&run_request, Utc::now()).await?;
        let other_run = repository.new_run(&run_request, Utc::now()).await?;

        let success_id = InvocationId::new();
        let failure_id = InvocationId::new();
        let new_batch = || {
            vec![
                DriverEvent::Invocation(InvocationEvent::InvocationSuccess(
                    InvocationSuccessEvent {
                        id: success_id,
                        parent: None,
                        predecessor: None,
                        function_name: "start".into(),
                        input: json!({}),
                        params: json!(null),
                        output: json!("ok"),
                        start_time: Utc::now(),
                        end_time: Utc::now(),
                    },
                )),
                DriverEvent::Invocation(InvocationEvent::InvocationFailure(
                    InvocationFailureEvent {
                        id: failure_id,
                        parent: Some(success_id),
                        predecessor: Some(success_id),
                        function_name: "next".into(),
                        input: json!("ok"),
                        params: json!(null),
                        start_time: Utc::now(),
                        error: Some(json!({ "message": "boom" })),
                    },
                )),
                DriverEvent::Log(LogEvent::Workflow(WorkflowLogMessage {
                    level: LogLevel::Info,
                    message: "hello".to_string(),
                    data: None,
                })),
            ]
        };

        // A batch for an unknown run is rejected with 404

        let unknown_run = RunId::new(999);
        let error =
            handle_events_batch(Path(unknown_run), State(state.clone()), Json(new_batch()))
                .await
                .expect_err("a batch for an unknown run should be rejected");
        assert_eq!(error.into_response().status(), StatusCode::NOT_FOUND);

        // A batch persists its invocation events under the run from the route

        handle_events_batch(Path(run), State(state.clone()), Json(new_batch()))
            .await
            .unwrap();
        let invocations = repository.get_invocations(run).await?;
        assert_eq!(invocations.len(), 2);
        assert!(invocations.iter().all(|invocation| invocation.run_id == run));
        let success = invocations
            .iter()
            .find(|invocation| invocation.id == success_id)
            .unwrap();
        assert!(success.end_time.is_some());
        let failure = invocations
            .iter()
            .find(|invocation| invocation.id == failure_id)
            .unwrap();
        assert!(failure.end_time.is_none());
        assert!(repository.get_invocations(other_run).await?.is_empty());

        // Re-sending the same batch does not duplicate invocations

        handle_events_batch(Path(run), State(state.clone()), Json(new_batch()))
            .await
            .unwrap();
        assert_eq!(repository.get_invocations(run).await?.len(), 2);

        // An empty batch is accepted

        handle_events_batch(Path(run), State(state), Json(Vec::new()))
            .await
            .unwrap();

        Ok(())
    }
}

use crate::{AppState, error::WorkflowCompletionError};
use axum::Json;
use axum::extract::{Path, State};
use axum::response::{IntoResponse, Response};
use florca_core::driver::DriverResult;
use florca_core::run::RunId;
use reqwest::StatusCode;
use std::sync::Arc;
use tracing::{debug, error};

#[axum::debug_handler]
pub async fn handle_complete_run(
    Path(run_id): Path<RunId>,
    State(state): State<Arc<AppState>>,
    Json(result): Json<DriverResult>,
) -> axum::response::Result<(), WorkflowCompletionError> {
    debug!(run = %run_id, "Received workflow completion");

    let run_exists = state
        .run_service
        .run_exists(run_id)
        .await
        .map_err(WorkflowCompletionError::Other)?;
    if !run_exists {
        return Err(WorkflowCompletionError::NotFound(run_id));
    }

    state
        .run_service
        .finalize_run(run_id, result)
        .await
        .map_err(WorkflowCompletionError::Other)?;

    Ok(())
}

impl IntoResponse for WorkflowCompletionError {
    fn into_response(self) -> Response {
        match self {
            WorkflowCompletionError::NotFound(_) => {
                (StatusCode::NOT_FOUND, self.to_string()).into_response()
            }
            WorkflowCompletionError::Other(err) => {
                error!("Driver event error: {}", err);
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error").into_response()
            }
        }
    }
}

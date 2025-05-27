use crate::{AppState, error::RunWorkflowError};
use axum::{
    Json,
    extract::{Path, State},
    response::{IntoResponse, Response},
};
use florca_core::run::{RunId, RunRequest};
use reqwest::StatusCode;
use serde_json::Value;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::error;

pub async fn run_workflow(
    State(state): State<Arc<RwLock<AppState>>>,
    Json(payload): Json<RunRequest>,
) -> axum::response::Result<Json<RunId>, RunWorkflowError> {
    let run_id = state.read().await.run_service.run_workflow(payload).await?;
    Ok(Json(run_id))
}

pub async fn invoke_child(
    Path(workflow_run): Path<RunId>,
    State(state): State<Arc<RwLock<AppState>>>,
    Json(invoke_child_args): Json<Value>,
) -> axum::response::Result<Json<Value>, RunWorkflowError> {
    let value = state
        .read()
        .await
        .run_service
        .invoke_child(workflow_run, invoke_child_args)
        .await?;
    Ok(value.into())
}

impl IntoResponse for RunWorkflowError {
    fn into_response(self) -> Response {
        match &self {
            RunWorkflowError::DeploymentNotFound(_) | RunWorkflowError::EntryPointNotFound(_) => {
                (StatusCode::NOT_FOUND, self.to_string()).into_response()
            }
            RunWorkflowError::Io(err) => {
                error!("{:?}", err);
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error").into_response()
            }
            RunWorkflowError::Other(err) => {
                error!("{:?}", err);
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error").into_response()
            }
        }
    }
}

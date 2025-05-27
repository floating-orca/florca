use crate::{AppState, error::PsError};
use axum::{
    Json,
    extract::State,
    response::{IntoResponse, Response},
};
use florca_core::ps::RunningWorkflow;
use reqwest::StatusCode;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::error;

pub async fn get_running_workflows(
    State(state): State<Arc<RwLock<AppState>>>,
) -> axum::response::Result<Json<Vec<RunningWorkflow>>, PsError> {
    let running_workflows = state
        .read()
        .await
        .ps_service
        .get_running_workflows()
        .await?;
    Ok(Json(running_workflows))
}

impl IntoResponse for PsError {
    fn into_response(self) -> Response {
        error!("{:?}", self.0);
        (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error").into_response()
    }
}

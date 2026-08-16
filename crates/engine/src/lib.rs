use anyhow::Result;
use inspection::InspectionService;
use kill::KillService;
use message::MessageService;
use ps::PsService;
use repository::SqlxEngineRepository;
use run::RunService;
use std::sync::Arc;
use tokio::signal::{self, unix::SignalKind};
use tracing::warn;

mod deployer_client;
pub mod driver;
mod driver_client;
mod error;
mod event_processor;
mod http;
pub mod inspection;
mod kill;
mod message;
pub mod process;
mod ps;
pub mod repository;
mod run;

pub use http::serve;

#[derive(Debug)]
pub struct AppState {
    pub ps_service: Arc<PsService>,
    pub message_service: Arc<MessageService>,
    pub kill_service: Arc<KillService>,
    pub inspection_service: Arc<InspectionService>,
    pub run_service: Arc<RunService>,
}

pub async fn init() -> Result<Arc<AppState>> {
    let engine_repository = Arc::new(SqlxEngineRepository::setup().await?);
    let process_manager = Arc::new(process::ProcessManager::new());

    let ps_service = Arc::new(ps::PsService::new(
        engine_repository.clone(),
        process_manager.clone(),
    ));
    let message_service = Arc::new(MessageService::new(process_manager.clone()));
    let kill_service = Arc::new(KillService::new(
        process_manager.clone(),
        engine_repository.clone(),
    ));
    let inspection_service = Arc::new(InspectionService::new(
        engine_repository.clone(),
        process_manager.clone(),
    ));
    let deployer_client = Arc::new(deployer_client::DeployerClientImpl);
    let run_service = Arc::new(RunService::new(
        process_manager.clone(),
        engine_repository.clone(),
        deployer_client.clone(),
    ));

    let state = AppState {
        ps_service,
        message_service,
        kill_service,
        inspection_service,
        run_service,
    };
    Ok(Arc::new(state))
}

/// Waits for a SIGTERM or SIGINT signal.
///
/// This function is intended to be used as a graceful shutdown signal handler.
///
/// # Panics
///
/// Panics if the signal handlers cannot be installed.
pub async fn shutdown_signal() {
    let mut terminate =
        signal::unix::signal(SignalKind::terminate()).expect("failed to install signal handler");
    let mut interrupt =
        signal::unix::signal(SignalKind::interrupt()).expect("failed to install signal handler");
    tokio::select! {
        _ = terminate.recv() => warn!("SIGTERM received, shutting down"),
        _ = interrupt.recv() => warn!("SIGINT received, shutting down"),
    }
}

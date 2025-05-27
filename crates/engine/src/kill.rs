use std::sync::Arc;

use anyhow::Result;
use florca_core::run::{AllOrRunId, RunId};
use tracing::warn;

use crate::{error::KillError, process::ProcessManager};

#[derive(Debug, Clone)]
pub struct KillService {
    process_manager: Arc<ProcessManager>,
}

impl KillService {
    pub fn new(process_manager: Arc<ProcessManager>) -> Self {
        KillService { process_manager }
    }

    pub async fn kill_runs(&self, all_or_run_id: AllOrRunId) -> Result<Vec<RunId>, KillError> {
        match all_or_run_id {
            AllOrRunId::All => {
                let drained = self
                    .process_manager
                    .driver_processes()
                    .write()
                    .await
                    .drain()
                    .collect::<Vec<_>>();
                let mut runs = Vec::new();
                for (run, driver_process) in drained {
                    crate::kill::kill_process_by_pid(driver_process.pid).await?;
                    runs.push(run);
                }
                Ok(runs)
            }
            AllOrRunId::RunId(run_id) => {
                let mut lock = self.process_manager.driver_processes().write().await;
                if let Some(driver_process) = lock.remove(&run_id) {
                    crate::kill::kill_process_by_pid(driver_process.pid).await?;
                } else {
                    return Err(KillError::NotFound(run_id));
                }
                Ok(vec![run_id])
            }
        }
    }
}

pub async fn kill_process_by_pid(pid: u32) -> Result<()> {
    let exit_status = tokio::process::Command::new("kill")
        .arg(pid.to_string())
        .status()
        .await?;
    match exit_status.code() {
        Some(0) => Ok(()),
        Some(1) => {
            warn!("No process found for pid {}", pid);
            Ok(())
        }
        _ => Err(anyhow::anyhow!("Could not kill process")),
    }
}

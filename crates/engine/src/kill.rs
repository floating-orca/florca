use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use florca_core::run::{AllOrRunId, RunId};
use tracing::warn;

use crate::repository::EngineRepository;
use crate::{error::KillError, process::ProcessManager};

const RUN_FINALIZATION_TIMEOUT: Duration = Duration::from_secs(5);

#[derive(Debug, Clone)]
pub struct KillService {
    process_manager: Arc<ProcessManager>,
    repository: Arc<dyn EngineRepository>,
}

impl KillService {
    pub fn new(
        process_manager: Arc<ProcessManager>,
        repository: Arc<dyn EngineRepository>,
    ) -> Self {
        KillService {
            process_manager,
            repository,
        }
    }

    /// Kills all runs and waits until they are finalized in the database, so
    /// that a shutdown does not leave the rows of killed runs open.
    pub async fn shutdown(&self) {
        let active = self.process_manager.registered_run_ids().await;
        if let Err(err) = self.kill_runs(AllOrRunId::All).await {
            warn!("Could not kill all driver processes: {err:#}");
        }
        let wait_for_finalization = async {
            loop {
                match self.repository.get_runs_without_end_time().await {
                    Ok(open_runs) => {
                        if !active
                            .iter()
                            .any(|run| open_runs.iter().any(|open| open.id == *run))
                        {
                            return;
                        }
                    }
                    Err(err) => {
                        warn!("Could not fetch open runs: {err:#}");
                        return;
                    }
                }
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
        };
        if tokio::time::timeout(RUN_FINALIZATION_TIMEOUT, wait_for_finalization)
            .await
            .is_err()
        {
            warn!("Timed out waiting for killed runs to be finalized");
        }
    }

    pub async fn kill_runs(&self, all_or_run_id: AllOrRunId) -> Result<Vec<RunId>, KillError> {
        match all_or_run_id {
            AllOrRunId::All => {
                let processes = self.process_manager.killable().await;

                // Runs are only removed from the map once their process is
                // killed, so a run whose kill failed can be killed again.
                let mut killed = Vec::new();
                let mut failed = Vec::new();
                for (run, pid) in processes {
                    match crate::kill::kill_process_by_pid(pid).await {
                        Ok(()) => killed.push(run),
                        Err(_) => failed.push(run),
                    }
                }
                for run in &killed {
                    self.process_manager.remove(*run).await;
                }
                if !failed.is_empty() {
                    let failed = failed
                        .iter()
                        .map(ToString::to_string)
                        .collect::<Vec<_>>()
                        .join(", ");
                    return Err(anyhow::anyhow!("Could not kill runs {failed}").into());
                }
                Ok(killed)
            }
            AllOrRunId::RunId(run_id) => {
                let driver_process = self
                    .process_manager
                    .get(run_id)
                    .await
                    .ok_or(KillError::NotFound(run_id))?;
                let pid = driver_process
                    .pid
                    .ok_or_else(|| anyhow::anyhow!("Run {run_id} is still starting"))?;
                crate::kill::kill_process_by_pid(pid).await?;
                self.process_manager.remove(run_id).await;
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
        _ => Err(anyhow::anyhow!("Could not kill process {pid}")),
    }
}

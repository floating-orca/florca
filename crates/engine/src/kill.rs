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

    /// Killing only signals the process. The run leaves the map when the
    /// engine sees the exit (`DriverManager`), so a killed run reports
    /// Running until it dies and a failed kill can be retried.
    pub async fn kill_runs(&self, all_or_run_id: AllOrRunId) -> Result<Vec<RunId>, KillError> {
        match all_or_run_id {
            AllOrRunId::All => {
                let processes = self.process_manager.killable().await;

                let mut killed = Vec::new();
                let mut failed = Vec::new();
                for (run, pid) in processes {
                    match crate::kill::kill_process_by_pid(pid).await {
                        Ok(()) => killed.push(run),
                        Err(_) => failed.push(run),
                    }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repository::UnusedRepository;

    #[tokio::test]
    async fn test_killed_run_stays_registered_until_its_process_exits() {
        let process_manager = Arc::new(ProcessManager::new());
        let service = KillService::new(process_manager.clone(), Arc::new(UnusedRepository));
        let run_id = RunId::new(1);

        let mut child = tokio::process::Command::new("sleep")
            .arg("60")
            .spawn()
            .unwrap();
        process_manager.register(run_id).await;
        process_manager
            .record_pid(run_id, child.id().unwrap())
            .await;

        service.kill_runs(AllOrRunId::RunId(run_id)).await.unwrap();
        assert!(process_manager.get(run_id).await.is_some());

        child.wait().await.unwrap();
    }
}

use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use florca_core::run::{AllOrRunId, RunId};
use tracing::warn;

use crate::repository::EngineRepository;
use crate::{error::KillError, process::ProcessManager};

const RUN_FINALIZATION_TIMEOUT: Duration = Duration::from_secs(5);

/// Longer than the driver's final flush retries, so a driver that can still
/// deliver its events is never force-killed while doing so
pub const KILL_ESCALATION_GRACE: Duration = Duration::from_mins(1);

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
            for (run, pid) in self.process_manager.pids().await {
                warn!("Sending SIGKILL to the driver of run {run}");
                let _ = signal_pid(pid, "KILL").await;
            }
        }
    }

    /// Killing only signals the process. The run leaves the map when the
    /// engine sees the exit (`DriverManager`), so a killed run reports
    /// Running until it dies and a failed kill can be retried. A run without
    /// a pid is killed by the `DriverManager` once its pid is recorded.
    pub async fn kill_runs(&self, all_or_run_id: AllOrRunId) -> Result<Vec<RunId>, KillError> {
        match all_or_run_id {
            AllOrRunId::All => {
                let processes = self.process_manager.mark_all_kill_requested().await;

                let mut killed = Vec::new();
                let mut failed = Vec::new();
                for (run, pid) in processes {
                    let Some(pid) = pid else {
                        // Killed by the DriverManager once the pid is recorded
                        killed.push(run);
                        continue;
                    };
                    match kill_and_escalate(self.process_manager.clone(), run, pid).await {
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
                    .mark_kill_requested(run_id)
                    .await
                    .ok_or(KillError::NotFound(run_id))?;
                if let Some(pid) = driver_process.pid {
                    kill_and_escalate(self.process_manager.clone(), run_id, pid).await?;
                }
                Ok(vec![run_id])
            }
        }
    }
}

/// Arms the SIGKILL backstop before signaling, so a requested kill cannot
/// miss it even when the TERM fails
pub async fn kill_and_escalate(
    process_manager: Arc<ProcessManager>,
    run_id: RunId,
    pid: u32,
) -> Result<()> {
    escalate_after_grace(process_manager, run_id, KILL_ESCALATION_GRACE);
    signal_pid(pid, "TERM").await
}

/// Force-kills the driver if the run is still in the map after the grace
/// period. Map presence means its process has not exited yet.
fn escalate_after_grace(
    process_manager: Arc<ProcessManager>,
    run_id: RunId,
    grace: Duration,
) {
    tokio::spawn(async move {
        tokio::time::sleep(grace).await;
        let Some(pid) = process_manager
            .get(run_id)
            .await
            .and_then(|driver_process| driver_process.pid)
        else {
            return;
        };
        warn!("Run {run_id} did not exit within the grace period, sending SIGKILL");
        if let Err(err) = signal_pid(pid, "KILL").await {
            warn!("Could not SIGKILL the driver of run {run_id}: {err:#}");
        }
    });
}

async fn signal_pid(pid: u32, signal: &str) -> Result<()> {
    let exit_status = tokio::process::Command::new("kill")
        .arg("-s")
        .arg(signal)
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

    async fn registered_child(
        process_manager: &ProcessManager,
        run_id: RunId,
    ) -> tokio::process::Child {
        let child = tokio::process::Command::new("sleep")
            .arg("60")
            .spawn()
            .unwrap();
        process_manager.register(run_id).await;
        process_manager
            .record_pid(run_id, child.id().unwrap())
            .await;
        child
    }

    #[tokio::test]
    async fn test_killed_run_stays_registered_until_its_process_exits() {
        let process_manager = Arc::new(ProcessManager::new());
        let service = KillService::new(process_manager.clone(), Arc::new(UnusedRepository));
        let run_id = RunId::new(1);

        let mut child = registered_child(&process_manager, run_id).await;
        service.kill_runs(AllOrRunId::RunId(run_id)).await.unwrap();
        assert!(process_manager.get(run_id).await.is_some());

        child.wait().await.unwrap();
    }

    #[tokio::test]
    async fn test_a_kill_before_the_pid_is_recorded_marks_the_run() {
        let process_manager = Arc::new(ProcessManager::new());
        let service = KillService::new(process_manager.clone(), Arc::new(UnusedRepository));
        let run_id = RunId::new(1);

        process_manager.register(run_id).await;
        service.kill_runs(AllOrRunId::RunId(run_id)).await.unwrap();

        let driver_process = process_manager.record_pid(run_id, 4242).await.unwrap();
        assert!(driver_process.kill_requested);
    }

    #[tokio::test]
    async fn test_kill_all_includes_runs_still_starting() {
        let process_manager = Arc::new(ProcessManager::new());
        let service = KillService::new(process_manager.clone(), Arc::new(UnusedRepository));
        let run_id = RunId::new(1);

        process_manager.register(run_id).await;
        let killed = service.kill_runs(AllOrRunId::All).await.unwrap();

        assert_eq!(killed, vec![run_id]);
        let driver_process = process_manager.record_pid(run_id, 4242).await.unwrap();
        assert!(driver_process.kill_requested);
    }

    #[tokio::test]
    async fn test_escalation_skips_a_run_that_already_exited() {
        let process_manager = Arc::new(ProcessManager::new());
        let run_id = RunId::new(1);

        let mut child = registered_child(&process_manager, run_id).await;
        process_manager.remove(run_id).await;

        escalate_after_grace(process_manager, run_id, Duration::from_millis(50));
        tokio::time::sleep(Duration::from_millis(200)).await;

        assert!(child.try_wait().unwrap().is_none());
        child.kill().await.unwrap();
    }

    #[tokio::test]
    async fn test_a_stuck_driver_is_force_killed_after_the_grace_period() {
        use std::os::unix::process::ExitStatusExt;

        let process_manager = Arc::new(ProcessManager::new());
        let run_id = RunId::new(1);

        let mut child = registered_child(&process_manager, run_id).await;
        escalate_after_grace(process_manager, run_id, Duration::from_millis(50));

        let status = child.wait().await.unwrap();
        assert_eq!(status.signal(), Some(9));
    }
}

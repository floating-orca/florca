use crate::{process::DriverProcess, process::ProcessManager, repository::EngineRepository};
use anyhow::Result;
use chrono::Utc;
use florca_core::{
    driver::{DriverErrorDetails, DriverResult, DriverResultKind},
    run::{RunId, RunRequest},
};
use serde_json::Value;
use std::{env, os::unix::process::ExitStatusExt, path::Path, process::ExitStatus, sync::Arc};
use tracing::{error, info};

mod driver_command;
mod driver_logs;

#[derive(Debug, Clone)]
pub struct DriverManager {
    run_id: RunId,
    process_manager: Arc<ProcessManager>,
    repository: Arc<dyn EngineRepository>,
}

impl DriverManager {
    pub fn new(
        run_id: RunId,
        process_manager: Arc<ProcessManager>,
        repository: Arc<dyn EngineRepository>,
    ) -> Self {
        Self {
            run_id,
            process_manager,
            repository,
        }
    }

    async fn add_pending_driver_process(&self, pid: u32) -> Result<()> {
        let mut lock = self.process_manager.driver_processes().write().await;
        lock.insert(self.run_id, DriverProcess { pid, port: None });
        Ok(())
    }

    async fn remove_driver_process(&self) -> Result<()> {
        self.process_manager
            .driver_processes()
            .write()
            .await
            .remove(&self.run_id);
        Ok(())
    }

    pub async fn run_workflow(
        self,
        run_request: RunRequest,
        temporary_directory_path: &Path,
    ) -> Result<()> {
        let outfile = tempfile::Builder::new()
            .prefix("run-")
            .suffix(".json")
            .tempfile()?;
        let command_result = self
            .run_driver(run_request, temporary_directory_path, outfile.path())
            .await?;
        self.remove_driver_process().await?;
        self.process_driver_process_result(outfile.path(), command_result)
            .await?;
        Ok(())
    }

    async fn run_driver(
        &self,
        run_request: RunRequest,
        temporary_directory_path: &Path,
        outfile_path: &Path,
    ) -> Result<ExitStatus> {
        let original_deno_lock_path = env::current_dir()?.join("deno.lock");
        let temporary_deno_lock_dir = tempfile::tempdir()?;
        let temporary_deno_lock_path = temporary_deno_lock_dir.path().join("deno.lock");
        tokio::fs::copy(&original_deno_lock_path, &temporary_deno_lock_path).await?;

        let mut command = driver_command::spawn_driver(
            run_request,
            temporary_directory_path,
            outfile_path,
            self.run_id,
            &temporary_deno_lock_path,
        )?;
        let pid = command.id().ok_or_else(|| {
            anyhow::anyhow!(
                "Failed to get PID for driver process with run ID {}",
                self.run_id
            )
        })?;
        self.add_pending_driver_process(pid).await?;

        let stdout = command.stdout.take().ok_or_else(|| {
            anyhow::anyhow!(
                "Failed to capture stdout for driver process with run ID {}",
                self.run_id
            )
        })?;
        let stderr = command.stderr.take().ok_or_else(|| {
            anyhow::anyhow!(
                "Failed to capture stderr for driver process with run ID {}",
                self.run_id
            )
        })?;
        driver_logs::parse_stdout_and_stderr(stdout, stderr, self.run_id);

        Ok(command.wait().await?)
    }

    async fn process_driver_process_result(
        &self,
        outfile_path: &Path,
        command_result: ExitStatus,
    ) -> Result<()> {
        let run_id = self.run_id;

        let driver_result = if let Some(15) = command_result.signal() {
            error!(run = run_id.to_string(), "Driver process was killed");
            DriverResult {
                run_id,
                result: DriverResultKind::Error(DriverErrorDetails {
                    kind: "DriverProcessKilled".to_string(),
                    message: "Driver process was killed".to_string(),
                }),
            }
        } else {
            read_driver_result(outfile_path).await.unwrap_or_else(|e| {
                error!(run = run_id.to_string(), "{}", e);
                DriverResult {
                    run_id,
                    result: DriverResultKind::Error(DriverErrorDetails {
                        kind: "ReadDriverResultError".to_string(),
                        message: e.to_string(),
                    }),
                }
            })
        };

        match driver_result.result {
            DriverResultKind::Success(success_details) => {
                self.finish_run(true, &success_details.value).await?;
                info!(run = run_id.to_string(), "Workflow run succeeded");
            }
            DriverResultKind::Error(error_details) => {
                self.finish_run(false, &serde_json::to_value(&error_details)?)
                    .await?;
                error!(run = run_id.to_string(), "Workflow run failed");
            }
        }

        Ok(())
    }

    async fn finish_run(&self, success: bool, output: &Value) -> Result<()> {
        self.repository
            .finish_run(success, self.run_id, output, Utc::now())
            .await?;
        Ok(())
    }
}

async fn read_driver_result(outfile_path: &Path) -> Result<DriverResult> {
    let s = tokio::fs::read_to_string(outfile_path).await?;
    let driver_result: DriverResult = serde_json::from_str(&s)?;
    Ok(driver_result)
}

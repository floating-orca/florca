use crate::process::ProcessManager;
use crate::repository::EngineRepository;
use anyhow::Result;
use florca_core::ps::RunningWorkflow;
use futures::{StreamExt, stream};
use std::{fmt::Debug, sync::Arc};

#[derive(Debug, Clone)]
pub struct PsService {
    repository: Arc<dyn EngineRepository>,
    process_manager: Arc<ProcessManager>,
}

impl PsService {
    pub fn new(
        repository: Arc<dyn EngineRepository>,
        process_manager: Arc<ProcessManager>,
    ) -> Self {
        PsService {
            repository,
            process_manager,
        }
    }
}

impl PsService {
    pub async fn get_running_workflows(&self) -> Result<Vec<RunningWorkflow>> {
        let running_workflows = stream::iter(self.repository.get_runs_without_end_time().await?)
            .filter_map(async |run| {
                self.process_manager
                    .driver_processes()
                    .read()
                    .await
                    .get(&run.id)
                    .map(|_p| RunningWorkflow {
                        run_id: run.id,
                        name: run.deployment_name,
                    })
            })
            .collect::<Vec<_>>()
            .await;
        Ok(running_workflows)
    }
}

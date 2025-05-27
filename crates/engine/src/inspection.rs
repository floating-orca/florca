use crate::process::ProcessManager;
use crate::repository::EngineRepository;
use crate::{error::GetInspectionError, repository::GetRunError};
use anyhow::{Context, Result};
use florca_core::inspection::{Inspection, InspectionEntry, RunStatus};
use florca_core::invocation::InvocationEntity;
use florca_core::run::{LatestOrRunId, RunEntity};
use std::collections::VecDeque;
use std::sync::Arc;

#[derive(Debug)]
pub struct InspectionService {
    repository: Arc<dyn EngineRepository>,
    process_manager: Arc<ProcessManager>,
}

impl InspectionService {
    pub fn new(
        repository: Arc<dyn EngineRepository>,
        process_manager: Arc<ProcessManager>,
    ) -> Self {
        Self {
            repository,
            process_manager,
        }
    }

    pub async fn get_inspection(
        &self,
        latest_or_run_id: LatestOrRunId,
    ) -> Result<Inspection, GetInspectionError> {
        let run = self.get_run(latest_or_run_id).await.map_err(|e| match e {
            GetRunError::NoLatest => GetInspectionError::NoLatest,
            GetRunError::NotFound(run_id) => GetInspectionError::NotFound(run_id),
            GetRunError::Other(error) => GetInspectionError::Other(error),
        })?;
        let inspection = self
            .build_inspection(run)
            .await
            .context("error building inspection")?;
        Ok(inspection)
    }

    async fn get_run(&self, latest_or_run_id: LatestOrRunId) -> Result<RunEntity, GetRunError> {
        match latest_or_run_id {
            LatestOrRunId::Latest => self.repository.get_latest_run().await,
            LatestOrRunId::RunId(run_id) => self.repository.get_run_by_id(run_id).await,
        }
    }

    async fn build_inspection(&self, run: RunEntity) -> Result<Inspection, GetInspectionError> {
        let status = self.status_of_run(&run).await?;
        let invocations = self.repository.get_invocations(run.id).await?;
        let mut root_entry: VecDeque<InspectionEntry> = VecDeque::new();
        let root_invocation = invocations
            .iter()
            .find(|invocation| invocation.parent.is_none() && invocation.predecessor.is_none());
        if let Some(root_invocation) = root_invocation {
            build_inspection_recursively(&invocations, root_invocation, &mut root_entry)?;
        }
        let inspection = Inspection::new(run, root_entry.into_iter().last(), status);
        Ok(inspection)
    }

    async fn status_of_run(&self, run: &RunEntity) -> Result<RunStatus> {
        let mut success = run.success;
        if success.is_none()
            && !self
                .process_manager
                .driver_processes()
                .read()
                .await
                .contains_key(&run.id)
        {
            success = Some(false);
        }
        let status = match success {
            Some(true) => RunStatus::Success,
            Some(false) => RunStatus::Error,
            None => RunStatus::Running,
        };
        Ok(status)
    }
}

fn build_inspection_recursively(
    invocations: &[InvocationEntity],
    invocation: &InvocationEntity,
    entries: &mut VecDeque<InspectionEntry>,
) -> Result<()> {
    let mut child_entries: VecDeque<InspectionEntry> = VecDeque::new();
    let children = invocations
        .iter()
        .filter(|l| l.parent.is_some_and(|p| p == invocation.id));
    for c in children {
        build_inspection_recursively(invocations, c, &mut child_entries)?;
    }

    let mut next_entry: VecDeque<InspectionEntry> = VecDeque::new();
    let next = invocations
        .iter()
        .find(|l| l.predecessor.is_some_and(|p| p == invocation.id));
    if let Some(n) = next {
        build_inspection_recursively(invocations, n, &mut next_entry)?;
    }
    let next = next_entry.pop_front().map(Box::new);
    let entry = InspectionEntry::new(invocation, child_entries.into_iter().collect(), next);
    entries.push_back(entry);
    Ok(())
}

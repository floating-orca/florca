use crate::process::ProcessManager;
use crate::repository::EngineRepository;
use crate::{
    error::GetInspectionError,
    repository::{GetLatestRunError, GetRunByIdError},
};
use anyhow::{Context, Result};
use florca_core::inspection::{Inspection, InspectionEntry, RunStatus};
use florca_core::invocation::InvocationEntity;
use florca_core::invocation::InvocationId;
use florca_core::run::{LatestOrRunId, RunEntity, RunId};
use std::collections::HashMap;
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
        let run = match latest_or_run_id {
            LatestOrRunId::Latest => self.load_latest_run().await?,
            LatestOrRunId::RunId(run_id) => self.load_run_by_id(run_id).await?,
        };
        let inspection = self
            .build_inspection(run)
            .await
            .context("error building inspection")?;
        Ok(inspection)
    }

    pub async fn get_status(&self, run_id: RunId) -> Result<RunStatus, GetInspectionError> {
        let run = self.load_run_by_id(run_id).await?;
        let status = self
            .status_of_run(&run)
            .await
            .context("error getting run status")?;
        Ok(status)
    }

    async fn load_latest_run(&self) -> Result<RunEntity, GetInspectionError> {
        self.repository
            .get_latest_run()
            .await
            .map_err(|error| match error {
                GetLatestRunError::NoLatest => GetInspectionError::NoLatest,
                GetLatestRunError::Other(error) => GetInspectionError::Other(error),
            })
    }

    async fn load_run_by_id(&self, run_id: RunId) -> Result<RunEntity, GetInspectionError> {
        self.repository
            .get_run_by_id(run_id)
            .await
            .map_err(|error| match error {
                GetRunByIdError::NotFound(run_id) => GetInspectionError::NotFound(run_id),
                GetRunByIdError::Other(error) => GetInspectionError::Other(error),
            })
    }

    async fn build_inspection(&self, run: RunEntity) -> Result<Inspection, GetInspectionError> {
        let status = self.status_of_run(&run).await?;
        let invocations = self.repository.get_invocations(run.id).await?;
        let root_entry = build_inspection_root(&invocations)?;
        let inspection = Inspection::new(run, root_entry, status);
        Ok(inspection)
    }

    async fn status_of_run(&self, run: &RunEntity) -> Result<RunStatus> {
        let mut success = run.success;
        if success.is_none() && !self.process_manager.is_registered(run.id).await {
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

fn build_inspection_root(invocations: &[InvocationEntity]) -> Result<Option<InspectionEntry>> {
    let mut by_id: HashMap<InvocationId, &InvocationEntity> =
        HashMap::with_capacity(invocations.len());
    let mut children_by_parent: HashMap<InvocationId, Vec<InvocationId>> = HashMap::new();
    let mut next_by_predecessor: HashMap<InvocationId, InvocationId> = HashMap::new();
    let mut root_invocation_id = None;

    for invocation in invocations {
        by_id.insert(invocation.id, invocation);

        if invocation.parent.is_none()
            && invocation.predecessor.is_none()
            && root_invocation_id.is_none()
        {
            root_invocation_id = Some(invocation.id);
        }

        if let Some(parent_id) = invocation.parent {
            children_by_parent
                .entry(parent_id)
                .or_default()
                .push(invocation.id);
        }

        if let Some(predecessor_id) = invocation.predecessor {
            next_by_predecessor.insert(predecessor_id, invocation.id);
        }
    }

    root_invocation_id
        .map(|invocation_id| {
            build_inspection_entry(
                invocation_id,
                &by_id,
                &children_by_parent,
                &next_by_predecessor,
            )
        })
        .transpose()
}

fn build_inspection_entry(
    invocation_id: InvocationId,
    by_id: &HashMap<InvocationId, &InvocationEntity>,
    children_by_parent: &HashMap<InvocationId, Vec<InvocationId>>,
    next_by_predecessor: &HashMap<InvocationId, InvocationId>,
) -> Result<InspectionEntry> {
    let invocation = by_id
        .get(&invocation_id)
        .copied()
        .context("missing invocation while building inspection graph")?;

    let child_entries = children_by_parent
        .get(&invocation_id)
        .into_iter()
        .flatten()
        .map(|child_id| {
            build_inspection_entry(*child_id, by_id, children_by_parent, next_by_predecessor)
        })
        .collect::<Result<Vec<_>>>()?;

    let next_entry = next_by_predecessor
        .get(&invocation_id)
        .map(|next_id| {
            build_inspection_entry(*next_id, by_id, children_by_parent, next_by_predecessor)
                .map(Box::new)
        })
        .transpose()?;

    Ok(InspectionEntry::new(invocation, child_entries, next_entry))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repository::UnusedRepository;
    use chrono::Utc;
    use florca_core::run::RunEntity;
    use serde_json::json;

    fn open_run(run_id: RunId) -> RunEntity {
        RunEntity {
            id: run_id,
            deployment_name: "test".into(),
            entry_point: "start".into(),
            input: json!({}),
            output: None,
            start_time: Utc::now(),
            end_time: None,
            success: None,
        }
    }

    #[tokio::test]
    async fn test_open_run_with_registered_driver_is_running() {
        let process_manager = Arc::new(ProcessManager::new());
        let service = InspectionService::new(Arc::new(UnusedRepository), process_manager.clone());
        let run_id = RunId::new(1);

        // Registered without a pid, as right after run creation
        process_manager.register(run_id).await;

        let status = service.status_of_run(&open_run(run_id)).await.unwrap();
        assert!(matches!(status, RunStatus::Running));
    }

    #[tokio::test]
    async fn test_open_run_without_registered_driver_is_error() {
        let process_manager = Arc::new(ProcessManager::new());
        let service = InspectionService::new(Arc::new(UnusedRepository), process_manager);

        let status = service.status_of_run(&open_run(RunId::new(1))).await.unwrap();
        assert!(matches!(status, RunStatus::Error));
    }
}

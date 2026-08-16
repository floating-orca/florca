use std::{collections::HashMap, sync::Arc};

use florca_core::run::RunId;
use tokio::sync::RwLock;

/// A run's driver process. Registered without a pid before the process is
/// spawned, so map presence already means "this run is alive".
#[derive(Debug, Clone)]
pub struct DriverProcess {
    pub pid: Option<u32>,
    pub port: Option<u16>,
}

#[derive(Debug, Clone)]
pub struct ProcessManager {
    driver_processes: Arc<RwLock<HashMap<RunId, DriverProcess>>>,
}

impl ProcessManager {
    #[must_use]
    pub fn new() -> Self {
        let driver_processes = Arc::new(RwLock::new(HashMap::new()));
        ProcessManager { driver_processes }
    }

    pub async fn register(&self, run_id: RunId) {
        self.driver_processes
            .write()
            .await
            .insert(run_id, DriverProcess {
                pid: None,
                port: None,
            });
    }

    pub async fn record_pid(&self, run_id: RunId, pid: u32) {
        self.driver_processes
            .write()
            .await
            .entry(run_id)
            .and_modify(|driver_process| driver_process.pid = Some(pid))
            .or_insert(DriverProcess {
                pid: Some(pid),
                port: None,
            });
    }

    /// Returns false if the run is not registered
    pub async fn record_port(&self, run_id: RunId, port: u16) -> bool {
        let mut lock = self.driver_processes.write().await;
        match lock.get_mut(&run_id) {
            Some(driver_process) => {
                driver_process.port = Some(port);
                true
            }
            None => false,
        }
    }

    pub async fn remove(&self, run_id: RunId) {
        self.driver_processes.write().await.remove(&run_id);
    }

    pub async fn get(&self, run_id: RunId) -> Option<DriverProcess> {
        self.driver_processes.read().await.get(&run_id).cloned()
    }

    pub async fn is_registered(&self, run_id: RunId) -> bool {
        self.driver_processes.read().await.contains_key(&run_id)
    }

    pub async fn registered_run_ids(&self) -> Vec<RunId> {
        self.driver_processes.read().await.keys().copied().collect()
    }

    /// Skips runs without a pid, which are still spawning their process
    pub async fn killable(&self) -> Vec<(RunId, u32)> {
        self.driver_processes
            .read()
            .await
            .iter()
            .filter_map(|(run, driver_process)| Some((*run, driver_process.pid?)))
            .collect()
    }

    pub async fn port_of(&self, run_id: RunId) -> Option<u16> {
        self.driver_processes
            .read()
            .await
            .get(&run_id)
            .and_then(|driver_process| driver_process.port)
    }
}

impl Default for ProcessManager {
    fn default() -> Self {
        Self::new()
    }
}

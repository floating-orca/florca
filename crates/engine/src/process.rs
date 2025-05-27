use std::{collections::HashMap, sync::Arc};

use florca_core::run::RunId;
use tokio::sync::RwLock;

#[derive(Debug, Clone)]
pub struct DriverProcess {
    pub pid: u32,
    pub port: Option<u16>,
}

#[derive(Debug, Clone)]
pub struct ProcessManager {
    pub driver_processes: Arc<RwLock<HashMap<RunId, DriverProcess>>>,
}

impl ProcessManager {
    #[must_use]
    pub fn new() -> Self {
        let driver_processes = Arc::new(RwLock::new(HashMap::new()));
        ProcessManager { driver_processes }
    }

    #[must_use]
    pub fn driver_processes(&self) -> &RwLock<HashMap<RunId, DriverProcess>> {
        &self.driver_processes
    }

    pub async fn get_port_for_run(&self, run_id: RunId) -> Option<u16> {
        self.driver_processes
            .read()
            .await
            .get(&run_id)
            .and_then(|x| x.port)
    }
}

impl Default for ProcessManager {
    fn default() -> Self {
        Self::new()
    }
}

use anyhow::Result;
use florca_core::function::KnFunctionConfig;
use florca_deployer::kn::{
    KnClient,
    kn_qualifier::{KnFunctionQualifier, KnUrl},
};
use std::{path::Path, sync::Arc};
use tokio::sync::RwLock;

#[derive(Debug)]
pub struct MockKnClient {
    pub functions: Arc<RwLock<Vec<KnFunctionQualifier>>>,
}

impl MockKnClient {
    pub fn new() -> Self {
        Self {
            functions: Arc::new(RwLock::new(vec![])),
        }
    }
}

#[async_trait::async_trait]
impl KnClient for MockKnClient {
    async fn create_kn_function(
        &self,
        kn_function_qualifier: &KnFunctionQualifier,
        _kn_function_config: &KnFunctionConfig,
        _implementation_path: &Path,
    ) -> Result<KnUrl> {
        let mut functions = self.functions.write().await;
        functions.push(kn_function_qualifier.clone());
        Ok(KnUrl(format!(
            "http://{kn_function_qualifier}.default.127.0.0.1.sslip.io/"
        )))
    }

    async fn find_deployed_kn_function(
        &self,
        kn_function_qualifier: &KnFunctionQualifier,
    ) -> Result<Option<KnUrl>> {
        let functions = self.functions.read().await;
        Ok(functions
            .iter()
            .find(|f| f == &kn_function_qualifier)
            .map(|f| KnUrl(format!("http://{f}.default.127.0.1.sslip.io/"))))
    }

    async fn delete_kn_function(&self, _kn_function_qualifier: &KnFunctionQualifier) -> Result<()> {
        let mut functions = self.functions.write().await;
        functions.retain(|f| f != _kn_function_qualifier);
        Ok(())
    }
}

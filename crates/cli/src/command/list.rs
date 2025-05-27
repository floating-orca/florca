use anyhow::Result;
use clap::Args;
use florca_core::{
    deployment::DeploymentName,
    http::{DeployerUrl, RequestBuilderExt},
};
use reqwest::blocking::Client;

#[derive(Debug, Args)]
pub struct ListCommand {}

impl ListCommand {
    /// # Errors
    ///
    /// This function will return an error if the request to the server fails, the server returns an error, or the response cannot be parsed.
    pub fn execute(self) -> Result<()> {
        let url = DeployerUrl::base();
        let response = Client::new().get(url).with_basic_auth_from_env().send()?;
        if let Err(e) = response.error_for_status_ref() {
            let text = response.text()?;
            if text.is_empty() {
                anyhow::bail!(e);
            }
            anyhow::bail!(text);
        }
        let deployments = response.json::<Vec<DeploymentName>>()?;
        println!("{}", serde_json::to_string_pretty(&deployments)?);
        Ok(())
    }
}

use anyhow::Result;
use clap::Args;
use florca_core::{http::EngineUrl, http::RequestBuilderExt, ps::RunningWorkflow};
use reqwest::blocking::Client;

#[derive(Debug, Args)]
pub struct PsCommand {}

impl PsCommand {
    /// # Errors
    ///
    /// This function will return an error if the request to the server fails, the server returns an error, or the response cannot be parsed.
    pub fn execute(self) -> Result<()> {
        let url = EngineUrl::base();
        let response = Client::new().get(url).with_basic_auth_from_env().send()?;
        if let Err(e) = response.error_for_status_ref() {
            let text = response.text()?;
            if text.is_empty() {
                anyhow::bail!(e);
            }
            anyhow::bail!(text);
        }
        let runs = response.json::<Vec<RunningWorkflow>>()?;
        println!("{}", serde_json::to_string_pretty(&runs)?);
        Ok(())
    }
}

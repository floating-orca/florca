use crate::util;
use crate::util::inspection::InspectDisplayOptions;
use anyhow::Result;
use clap::Args;
use florca_core::deployment::DeploymentName;
use florca_core::function::FunctionName;
use florca_core::http::{EngineUrl, RequestBuilderExt};
use florca_core::run::RunRequest;
use florca_core::run::{LatestOrRunId, RunId};
use reqwest::blocking::{Client, Response};
use serde_json::Value;
use std::{thread, time::Duration};

#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Args)]
pub struct RunCommand {
    /// The name of the deployment to run
    #[arg(short, long)]
    pub deployment_name: DeploymentName,

    /// The input to the workflow (JSON)
    #[arg(short, long, value_parser = util::parse_json)]
    pub input: Option<Value>,

    /// The entry point to run
    #[arg(short, long, default_value = "start")]
    pub entry_point: FunctionName,

    /// The params to pass to the workflow (JSON)
    #[arg(short, long, value_parser = util::parse_json)]
    pub params: Option<Value>,

    /// Wait for the workflow to finish and show the result
    #[arg(short, long)]
    pub wait: bool,

    /// Show inputs to functions (requires --wait)
    #[arg(long, requires = "wait")]
    pub show_inputs: bool,

    /// Show params passed to functions (requires --wait)
    #[arg(long, requires = "wait")]
    pub show_params: bool,

    /// Show outputs from functions (requires --wait)
    #[arg(long, requires = "wait")]
    pub show_outputs: bool,
}

impl RunCommand {
    /// # Errors
    ///
    /// This function will return an error in the following cases:
    ///
    /// * The run request to the server fails, the server returns an error, or the response cannot be parsed
    /// * The inspection data cannot be retrieved or parsed while waiting for the run to finish
    pub fn execute(self) -> Result<()> {
        let response = request_run(&self)?;
        if let Err(e) = response.error_for_status_ref() {
            let text = response.text()?;
            if text.is_empty() {
                anyhow::bail!(e);
            }
            anyhow::bail!(text);
        }
        let run_id = response.json::<RunId>()?;
        println!("Run: {}", &run_id);
        println!("{}", EngineUrl::path(&[&run_id.to_string()]));
        if self.wait {
            loop {
                let inspection = util::inspection::get_inspection(&LatestOrRunId::RunId(run_id))?;
                if !inspection.workflow_is_running() {
                    let inspect_options = InspectDisplayOptions {
                        show_inputs: self.show_inputs,
                        show_params: self.show_params,
                        show_outputs: self.show_outputs,
                    };
                    util::inspection::print_inspection(&inspection, &inspect_options);
                    break;
                }
                thread::sleep(Duration::from_secs(1));
            }
        }
        Ok(())
    }
}

fn request_run(run_args: &RunCommand) -> Result<Response> {
    let run_request = RunRequest {
        deployment_name: run_args.deployment_name.clone(),
        entry_point: run_args.entry_point.clone(),
        input: run_args.input.clone().unwrap_or(Value::Null),
        params: run_args.params.clone().unwrap_or(Value::Null),
    };
    let url = EngineUrl::base();
    let response = Client::new()
        .post(url)
        .with_basic_auth_from_env()
        .json(&run_request)
        .send()?;
    Ok(response)
}

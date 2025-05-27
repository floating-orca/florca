use crate::util;
use crate::util::inspection::InspectDisplayOptions;
use anyhow::Result;
use clap::Args;
use florca_core::run::{LatestOrRunId, RunId};

#[derive(Debug, Args)]
pub struct InspectCommand {
    #[command(flatten)]
    pub latest_or_run_id_args: LatestOrRunIdArgs,

    /// Show inputs to functions
    #[arg(short('i'), long)]
    pub show_inputs: bool,

    /// Show params passed to functions
    #[arg(short('p'), long)]
    pub show_params: bool,

    /// Show outputs from functions
    #[arg(short('o'), long)]
    pub show_outputs: bool,
}

#[derive(Debug, Args)]
#[group(required = true, multiple = false)]
pub struct LatestOrRunIdArgs {
    /// Inspect the latest run
    #[arg(short, long)]
    pub latest: bool,

    /// The ID of a specific run to inspect
    pub run_id: Option<RunId>,
}

impl From<LatestOrRunIdArgs> for LatestOrRunId {
    fn from(args: LatestOrRunIdArgs) -> Self {
        if args.latest {
            LatestOrRunId::Latest
        } else {
            LatestOrRunId::RunId(args.run_id.unwrap())
        }
    }
}

impl InspectCommand {
    /// # Errors
    ///
    /// This function will return an error if the inspection data cannot be retrieved or parsed.
    pub fn execute(self) -> Result<()> {
        let latest_or_run_id = LatestOrRunId::from(self.latest_or_run_id_args);
        let inspection = util::inspection::get_inspection(&latest_or_run_id)?;
        println!("Run: {}", inspection.run_id);
        if inspection.workflow_is_running() {
            println!("Status: running");
        } else {
            let inspect_options = InspectDisplayOptions {
                show_inputs: self.show_inputs,
                show_params: self.show_params,
                show_outputs: self.show_outputs,
            };
            util::inspection::print_inspection(&inspection, &inspect_options);
        }
        Ok(())
    }
}

use anyhow::Result;
use clap::Args;
use florca_core::http::{EngineUrl, RequestBuilderExt};
use florca_core::run::{AllOrRunId, RunId};
use itertools::Itertools;
use reqwest::blocking::Client;

#[derive(Debug, Args)]
pub struct KillCommand {
    #[command(flatten)]
    pub all_or_run_id_args: AllOrRunIdArgs,
}

#[derive(Debug, Args)]
#[group(required = true, multiple = false)]
pub struct AllOrRunIdArgs {
    /// Kill all runs
    #[arg(short, long)]
    pub all: bool,

    /// The ID of a specific run to kill
    pub run_id: Option<RunId>,
}

impl From<AllOrRunIdArgs> for AllOrRunId {
    fn from(args: AllOrRunIdArgs) -> Self {
        if args.all {
            AllOrRunId::All
        } else {
            AllOrRunId::RunId(args.run_id.unwrap())
        }
    }
}

impl KillCommand {
    /// # Errors
    ///
    /// This function will return an error if the request to the server fails, the server returns an error, or the response cannot be parsed.
    pub fn execute(self) -> Result<()> {
        let all_or_run_id = AllOrRunId::from(self.all_or_run_id_args);
        let url = EngineUrl::path(&[&all_or_run_id.to_string()]);
        let response = Client::new()
            .delete(url)
            .with_basic_auth_from_env()
            .send()?;
        if let Err(e) = response.error_for_status_ref() {
            let text = response.text()?;
            if text.is_empty() {
                anyhow::bail!(e);
            }
            anyhow::bail!(text);
        }
        let runs = response.json::<Vec<RunId>>()?;
        print_killed_runs(&runs);
        Ok(())
    }
}

fn print_killed_runs(runs: &[RunId]) {
    match runs {
        [] => {
            println!("No runs killed");
        }
        [run] => {
            println!("Killed run {run}");
        }
        _ => {
            println!("Killed runs {}", runs.iter().sorted().join(", "));
        }
    }
}

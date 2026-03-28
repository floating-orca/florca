use crate::command::{
    CompletionsCommand, DeleteCommand, DeployCommand, InfoCommand, InspectCommand, KillCommand,
    ListCommand, MessageCommand, NewCommand, PsCommand, RunCommand, TemplatesCommand,
};
use crate::util;
use clap::{Args, Parser, Subcommand};
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(name= "florca", version, about = "A command-line interface for florca", long_about = None)]
pub struct Cli {
    #[command(flatten)]
    pub global_opts: GlobalOpts,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Args)]
pub struct GlobalOpts {
    /// An optional .env file to load in addition to .env and .env.local
    #[arg(long, value_parser = util::validate_path_exists)]
    pub env_file: Option<PathBuf>,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Generate shell completions
    Completions(CompletionsCommand),
    /// Delete a deployment
    Delete(DeleteCommand),
    /// Deploy a workflow
    Deploy(DeployCommand),
    /// Get information about the CLI
    Info(InfoCommand),
    /// Inspect a workflow run
    Inspect(InspectCommand),
    /// Kill a workflow run
    Kill(KillCommand),
    /// List deployments
    List(ListCommand),
    /// Interact with workflow message handlers
    Message(MessageCommand),
    /// Create a new function
    New(NewCommand),
    /// List running workflows
    Ps(PsCommand),
    /// Run a workflow
    Run(RunCommand),
    /// List available templates
    Templates(TemplatesCommand),
}

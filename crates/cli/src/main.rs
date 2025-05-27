use anyhow::Result;
use clap::Parser;
use florca::cli::Cli;

fn main() -> Result<()> {
    let cli = Cli::parse();
    if let Some(env_file) = &cli.global_opts.env_file {
        dotenvy::from_path(env_file).ok();
    }
    dotenvy::from_filename(".env.local").ok();
    dotenvy::dotenv().ok();
    florca::run(cli)
}

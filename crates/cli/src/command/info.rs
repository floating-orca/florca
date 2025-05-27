use clap::Args;
use florca_core::http::{BasicAuth, DeployerUrl, EngineUrl};

#[derive(Debug, Args)]
pub struct InfoCommand {}

impl InfoCommand {
    pub fn execute(self) {
        println!("Basic Auth Username: {}", BasicAuth::from_env().username);
        println!(
            "Basic Auth Password: {}",
            if BasicAuth::from_env().password.is_some() {
                "[REDACTED]"
            } else {
                "[NOT SET]"
            }
        );
        println!("Deployer URL: {}", DeployerUrl::base());
        println!("Engine URL: {}", EngineUrl::base());
    }
}

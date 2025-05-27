use crate::Cli;
use clap::{Args, CommandFactory};
use clap_complete::Shell;

#[derive(Debug, Args)]
pub struct CompletionsCommand {
    /// The shell to generate completions for
    #[arg(value_enum)]
    pub shell: Shell,
}

impl CompletionsCommand {
    pub fn execute(self) {
        let command = &mut Cli::command();
        clap_complete::generate(
            self.shell,
            command,
            command.get_name().to_string(),
            &mut std::io::stdout(),
        );
    }
}

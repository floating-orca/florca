use crate::util;
use clap::Args;

#[derive(Debug, Args)]
pub struct TemplatesCommand {}

impl TemplatesCommand {
    pub fn execute(self) {
        for (provider, templates) in &util::template::available_function_templates() {
            println!("Provider: {}", provider.code());
            for t in templates {
                println!("  - {t}");
            }
        }
    }
}

use anyhow::Result;
use clap::Args;

use super::{Action, Context, RunType};

#[derive(Debug, Args)]
pub struct ShowStatus;

impl Action for ShowStatus {
    fn run(self, context: Context) -> Result<RunType> {
        println!("{}", self.get_status(context));

        Ok(RunType::Completed)
    }
}

impl ShowStatus {
    fn get_status(&self, context: Context) -> String {
        let mut status_text = String::from("\n    - logged in to Checkvist: \t");
        match &context.api_token {
            Some(_) => {
                status_text.push('✅');
            }
            None => {
                status_text.push('❌');
            }
        }

        status_text.push('\n');
        status_text.push_str("    - default list: \t\t");
        match &context.config {
            Some(config) => {
                status_text.push_str(&config.list_name);
            }
            None => {
                status_text.push('❌');
            }
        }
        status_text.push('\n');

        status_text
    }
}

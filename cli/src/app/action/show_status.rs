use anyhow::Result;
use clap::Args;

use super::{Action, RunType};
use crate::app::context;

#[derive(Debug, Args)]
pub struct ShowStatus;

impl Action for ShowStatus {
    fn run(self, context: context::Context) -> Result<RunType> {
        Ok(RunType::Completed(self.get_status(context)))
    }
}

impl ShowStatus {
    fn get_status(&self, context: context::Context) -> String {
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
            Ok(config) => {
                status_text.push_str(&config.list_name);
                status_text.push('\n');
                match &config.bookmarks {
                    Some(bookmarks) => {
                        status_text.push_str(&format!("    - bookmarks \t\t✅ {:?}", bookmarks))
                    }
                    None => status_text.push_str("    - bookmarks: \t\t❌"),
                };
            }
            Err(_) => {
                status_text.push('❌');
                status_text.push_str("\n    - bookmarks:\t\t❌");
            }
        }
        status_text.push('\n');

        status_text
    }
}

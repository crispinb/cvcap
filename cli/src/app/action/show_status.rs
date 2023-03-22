use anyhow::Result;
use bpaf::{command, construct, parsers::ParseCommand, pure, Parser};

use super::{Action, RunType};
use crate::app::{cli::Command, context};

#[derive(Debug, Clone)]
pub struct ShowStatus;

impl Action for ShowStatus {
    fn run(self, context: context::Context) -> Result<RunType> {
        Ok(RunType::Completed(self.get_status(context)))
    }
}

impl ShowStatus {
    pub fn command() -> ParseCommand<Command> {
        let status_action = pure(ShowStatus);
        let status = construct!(Command::ShowStatus(status_action))
            .to_options()
            .descr("Check whether cvcap is logged in to Checkvist, and has a default list and/or bookmark(s)");
        command("status", status).help("Check cvcap status: whether logged in and has default list and/or bookmark")
    }

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
                        let bookmark_display = bookmarks
                            .iter()
                            .map(|b| b.to_string())
                            .collect::<Vec<String>>()
                            .join(", ");
                        status_text.push_str(&format!(
                            "    - bookmarks ({}):\t\t{}",
                            bookmarks.len(),
                            bookmark_display
                        ))
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

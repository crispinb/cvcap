use anyhow::Result;
use bpaf::{command, construct, parsers::ParseCommand, pure, Parser};

use super::{Action, RunType};
use crate::app::{action, cli::Command, context, creds};

#[derive(Debug, Clone)]
pub struct LogOut;

impl LogOut {
    pub fn command() -> ParseCommand<Command> {
        let logout_action = pure(action::LogOut);
        let logout = construct!(Command::LogOut(logout_action))
            .to_options()
            .descr("Remove all login data for the logged in user");
        command("logout", logout).help("Removes all login data for the logged in user")
    }
}

impl Action for LogOut {
    fn run(self, context: context::Context) -> Result<RunType> {
        let msg = if context.api_token.is_some() {
            creds::delete_api_token(&context.keychain_service_name)?;
            "cvcap is now logged out"
        } else {
            "cvcap is already logged out"
        };

        Ok(RunType::Completed(msg.into()))
    }
}

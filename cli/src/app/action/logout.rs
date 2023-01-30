use anyhow::Result;
use clap::Args;

use super::{Action, RunType};
use crate::app::{context, creds};

#[derive(Debug, Args)]
pub struct LogOut;

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

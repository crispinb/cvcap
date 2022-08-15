use crate::app::{
    action::{self, Action},
    creds, Context,
};
use anyhow::Result;
use clap::Args;

#[derive(Debug, Args)]
pub struct LogOut;

impl Action for LogOut {
    fn run(self, context: crate::app::Context) -> Result<action::RunType> {
        if context.api_token.is_some() {
            creds::delete_api_token()?;
            println!("cvcap is now logged out");
        } else {
            println!("cvcap is already logged out")
        }

        Ok(action::RunType::Completed)
    }
}

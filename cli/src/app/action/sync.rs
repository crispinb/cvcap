use anyhow::{anyhow, Result};
use clap::Args;
use log::error;

use cvapi::{sqlite::SqliteStore, sqlite::SqliteSyncClient, ApiClient};

use super::{Action, RunType};
use crate::app;
use crate::progress_indicator::ProgressIndicator;

#[derive(Debug, Args)]
pub struct Sync;

/// Temp copypasta from Add, to test fake sync in cli
impl Action for Sync {
    fn run(self, context: crate::app::Context) -> Result<RunType> {
        let api_token = match context.api_token {
            Some(token) => token,
            None => self.login_user(context.run_interactively)?,
        };
        let api_client = ApiClient::new(
            "https://checkvist.com/".into(),
            api_token,
            // clippy warns about the unit argument, but I want it for the side effect
            #[allow(clippy::unit_arg)]
            |token| {
                app::creds::save_api_token_to_keyring(token)
                    .unwrap_or(error!("Couldn't save token to keyring"))
            },
        );

        let store = SqliteStore::init_with_file(&app::config::config_dir().join("data.db"))?;
        let client = SqliteSyncClient::new(api_client, store);

        ProgressIndicator::new('.', Box::new(|| println!("Syncing lists")), 250)
            .run(|| client.sync_lists().map_err(|e| anyhow!(e)))?;
        println!("\nDone");

        Ok(RunType::Completed)
    }
}

impl Sync {
    fn login_user(&self, is_interactive: bool) -> Result<String> {
        if is_interactive {
            app::creds::login_user()
        } else {
            Err(anyhow!(app::Error::LoggedOut))
        }
    }
}

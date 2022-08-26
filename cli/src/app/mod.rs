mod action;
mod cli;
mod config;
pub mod creds;

use std::fmt;

pub use action::{Action, Add, LogOut, RunType, ShowStatus};
pub use cli::{Cli, Command};
pub use config::Config;

pub struct Context {
    pub config: Option<Config>,
    pub api_token: Option<String>,
    pub run_interactively: bool,
}

/// App errors are largely handled by creating anyhow::Errors
/// But where we do need to take action depending on the concrete error,
/// add a variant here.
/// If wrapped in anyhow::Errors, they can be retrieved with anyhow::root_cause()
#[derive(Debug)]
pub enum Error {
    MissingPipe,
    LoggedOut,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Error::MissingPipe => {
                write!(f, "Tried to read from stdin pipe, but nothing was piped")
            }
            Error::LoggedOut => {
                write!(f, "cvcap is logged out")
            }
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}

pub mod config;
pub mod context;
pub mod creds;

mod action;
mod bookmark;
mod cli;

use std::fmt;

pub use action::{Action, Add, LogOut, RunType, ShowStatus};
pub use cli::{Cli, Command};
pub use config::Config;

/// App errors are largely handled by creating anyhow::Errors
/// But where we do need to take action depending on the concrete error,
/// add a variant here.
/// If wrapped in anyhow::Errors, they can be retrieved with anyhow::root_cause()
#[derive(Debug)]
pub enum Error {
    MissingPipe,
    LoggedOut,
    BookmarkMissingError(String),
    BookmarkFormatError,
    InvalidConfigFile(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::MissingPipe => {
                write!(f, "Tried to read from stdin pipe, but nothing was piped")
            }
            Error::LoggedOut => {
                write!(f, "cvcap is logged out")
            }
            Error::BookmarkFormatError => {
                write!(f, "Bad bookmark format")
            }
            Error::BookmarkMissingError(name) => {
                write!(f, "No bookmark named {} found", name)
            }
            Error::InvalidConfigFile(path) => {
                write!(f, "The cvcap config file {} could not be understood", path)
            }
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}

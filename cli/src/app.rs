pub mod bookmark;
pub mod config;
pub mod context;
pub mod creds;

mod action;
mod cli;
mod interaction;

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
    // adding the MiscError temporarily to help with the distributed error problem.
    // But I think we're going to dump it for plain anyhow errors
    // (actually as per  my original scheme - anyhow for ordinary,
    // named errors for special handling)
    // NOTE that the special handling might not only be reporting in main.
    // - those that need special handling somewhere
    // - those that need special handling somewhere, but plain reporting
    //          ah but of course we can just allow these to go to the generic
    //          error reporting case in main! They don't need to be different.
    // - those that need plain reporting - either this MiscType, or an anyhow type if I can find a
    // way to tag anyhow errors
    // - those that need plain reporting but AS AN UNEXPECTED ERROR
    MiscError(String),
    MissingPipe,
    LoggedOut,
    BookmarkMissing(String),
    InvalidBookmarkStringFormat,
    BookmarkInvalid,
    InvalidConfigFile(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::MiscError(msg) => {
                write!(f, "{}", msg)
            }
            Error::MissingPipe => {
                write!(f, "Tried to read from stdin pipe, but nothing was piped")
            }
            Error::LoggedOut => {
                write!(f, "cvcap is logged out")
            }
            Error::InvalidBookmarkStringFormat => {
                write!(f, "Bad bookmark format")
            }
            Error::BookmarkMissing(name) => {
                write!(f, "No bookmark named {} found", name)
            }
            Error::BookmarkInvalid => {
                write!(f, "This location isn't known to Checkvist")
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

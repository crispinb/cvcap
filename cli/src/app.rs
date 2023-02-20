pub mod bookmark;
pub mod config;
pub mod context;
pub mod creds;

mod action;
mod cli;
mod interaction;

use std::fmt;

pub use action::{Action, AddTaskCommand, LogOut, RunType, ShowStatus};
pub use cli::{Cli, Command};
pub use config::Config;

/// App maintains 3 varieties of errors:
/// Generic reportable errors -
///     ie. Error::Reportable
///     Can use anyhow::Context
///     This can be added via its Result impl, so we don't need to construt
///     an error - just use .[with_]context("message")?;
///     These are expected, and are simply reported to the user
///     on stderr
///     eg. that there is no config file is a predictable occurence
/// Errors requiring special handling
///     Error::[all other variants]
/// TODO: add example
/// Unexpected or unhandled errors
///    anyhow::Error (ie. as constructed with `anyhow!` macro)
///    These are reported to the user as unexpected, with instructions
///    about how to report as an issue
#[derive(Debug)]
pub enum Error {
    /// Expected errors - ie. those that can just be written to
    /// user on stderr. No special handling, and no hints to the user
    /// that this is a bug
    Reportable(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Reportable(msg) => {
                write!(f, "{}", msg)
            }
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}

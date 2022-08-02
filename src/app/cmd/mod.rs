mod add;
mod show_status;
pub use self::add::Add;
pub use self::show_status::ShowStatus;
use crate::app::Context;
use anyhow::Result;
use std::fmt;

pub trait Action {
    fn run(self, context: Context) -> Result<RunType>;
}

pub enum RunType {
    Completed,
    Cancelled,
}

/// Cmd errors are largely handled by creating anyhow::Errors
/// But if we need to take action depending on the concrete error,
/// wrap an appropriate cmd::Error with the anyhow! macro
/// The cmd::Error can be retrieved with anyhow::Error::root_cause
#[derive(Debug)]
pub enum Error {
    MissingPipe,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Error::MissingPipe => {
                write!(f, "Tried to read from stdin pipe, but nothing was piped")
            }
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}

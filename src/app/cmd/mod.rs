mod add;
mod logout;
mod show_status;
pub use self::add::Add;
pub use self::logout::LogOut;
pub use self::show_status::ShowStatus;
use crate::app::Context;
use anyhow::Result;

pub trait Action {
    fn run(self, context: Context) -> Result<RunType>;
}

// TODO: is this effectively the same as std::ops::ControlFlow?
pub enum RunType {
    Completed,
    Cancelled,
}

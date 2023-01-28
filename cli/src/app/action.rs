mod add;
mod add_bookmark;
mod logout;
mod show_status;

pub use self::add::Add;
pub use self::logout::LogOut;
pub use self::show_status::ShowStatus;

use anyhow::Result;

use crate::app::context;

pub trait Action {
    fn run(self, context: context::Context) -> Result<RunType>;
}

pub enum RunType {
    Completed,
    Cancelled,
}

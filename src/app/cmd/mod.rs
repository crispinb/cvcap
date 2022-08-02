mod add;
mod show_status;
mod logout;
pub use self::add::Add;
pub use self::show_status::ShowStatus;
pub use self::logout::LogOut;
use crate::app::Context;
use anyhow::Result;

pub trait Action {
    fn run(self, context: Context) -> Result<RunType>;
}

pub enum RunType {
    Completed,
    Cancelled,
}

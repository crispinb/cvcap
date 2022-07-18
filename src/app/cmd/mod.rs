mod add;
mod show_status;
pub use self::add::Add;
pub use self::show_status::ShowStatus;
use crate::app::Context;
use anyhow::Result;

pub trait Action {
    fn run(self, context: Context) -> Result<RunType>;
}

pub enum RunType {
    Completed,
    Cancelled
}

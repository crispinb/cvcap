mod add;
mod logout;
mod show_status;
mod sync;

use anyhow::Result;

pub use self::add::Add;
pub use self::logout::LogOut;
pub use self::show_status::ShowStatus;
pub use self::sync::Sync;
use crate::app::Context;

pub trait Action {
    fn run(self, context: Context) -> Result<RunType>;
}

pub enum RunType {
    Completed,
    Cancelled,
}

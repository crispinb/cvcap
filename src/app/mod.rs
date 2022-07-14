pub mod cmd;
pub mod config;
pub mod creds;
pub use config::Config;

pub struct Context {
    pub config: Option<Config>,
    pub api_token: Option<String>,
}

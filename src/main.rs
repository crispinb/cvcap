#[allow(unused)]
mod app;
mod progress_indicator;
use anyhow::{Error, Result};
use app::{
    cmd::{self, Action},
    creds, Config,
};
use clap::{Parser, Subcommand};
use cvcap::CheckvistError;
use env_logger::Env;
use log::{error, info};

// TODO: reinstate -v (not currently working)
// TODO - make 'add' action the default when no action is supplied

// Logging.
// Convention: reserve trace and debug levels for libraries (eg. checkvist api)
// Levels used in executable:
// - error: any non-recoverable error (eg. inability to parse config toml: can recover by o662xkDtJuGaFa2verwriting)
// - warn: recoverable errors
// - info: transient info for debugging

const BANNER: &str = r"                           
  _   _   _   _   _  
 / \ / \ / \ / \ / \ 
( c | v | c | a | p )
 \_/ \_/ \_/ \_/ \_/ 
                              

";

#[derive(Parser, Debug)]
#[clap(version, name=BANNER, about = "A minimal cli capture tool for Checkvist (https://checkvist.com)")]
#[clap(args_conflicts_with_subcommands = true, arg_required_else_help = true)]
struct Cli {
    #[clap(subcommand)]
    subcommand: Command,
    /// Enable verbose logging. In case of trouble
    #[clap(short = 'v', long = "verbose")]
    verbose: bool,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// (Default) Add a new task to your default list (you'll be prompted if you don't have one yet)
    Add(cmd::Add),
    /// Check whether cvcap is logged in, and if it has a default list set
    #[clap(name = "status")]
    ShowStatus(cmd::ShowStatus),
}

impl cmd::Action for Command {
    fn run(self, context: app::Context) -> Result<cmd::RunType> {
        match self {
            Command::Add(add) => add.run(context),
            Command::ShowStatus(cmd) => cmd.run(context),
        }
    }
}

fn main() {
    let cli = Cli::parse();
    let context = app::Context {
        config: Config::read_from_file(),
        api_token: creds::get_api_token_from_keyring(),
    };

    let log_level = if cli.verbose { "DEBUG" } else { "OFF" };
    env_logger::Builder::from_env(Env::default().default_filter_or(log_level)).init();

    match cli.subcommand.run(context) {
        Err(err) => {
            error!("Fatal error. Cause: {:?}", err.root_cause());
            display_error(err);
            std::process::exit(1);
        }
        Ok(cmd::RunType::Continued) => (),
        Ok(cmd::RunType::Cancelled) => println!("Cancelled"),
    }
    std::process::exit(0);
}

fn display_error(err: Error) {
    match err.root_cause().downcast_ref() {
        Some(CheckvistError::TokenRefreshFailedError) => {
            eprint_logged_out();
            match creds::delete_api_token() {
                Err(err) => error!("Something went wrong deleting invalid api token: {}", err),
                _ => info!("Expired api token was deleted"),
            }
        }
        _ => {
            let err = err;
            eprint_error(err);
        }
    }
}

#[inline(always)]
fn eprint_logged_out() {
    eprintln!(
        r#"
    You have been logged out of the Checkvist API.
    Please run cvcap again to log back in"#
    );
}

#[inline(always)]
fn eprint_error(err: Error) {
    eprintln!(
        r#"
    Error: {}

    If you want to report this, fill out an issue at 
    {}.
    To gather more details that might help solve issue, 
    run the same command again with the '-v' switch,
    and copy the output into the issue.
            "#,
        err, "https://github.com/crispinb/cvcap/issues"
    )
}

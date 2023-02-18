use anyhow::Error;
use clap::Parser;
use env_logger::Env;
use log::{error, info};

use cvapi::CheckvistError;
use cvcap::colour_output::{ColourOutput, StreamKind, Style};
use cvcap::{creds, Action, Cli, Command, Error as AppError, RunType};

// Logging.
// Convention: reserve trace and debug levels for libraries (eg. checkvist api)
// Levels used in executable:
// - error: any non-recoverable error (eg. inability to parse config toml)
// - warn: recoverable errors
// - info: transient info for debugging

// Exit codes
// 0 - success
// 1 - any error generated by cvcap
// 2 - clap parsing errors

fn main() {
    let cli = Cli::parse();
    // if no subcommand is provided, create a default 'add', with task content from first arg
    let subcommand = cli
        .subcommand
        .unwrap_or_else(|| Command::default(&cli.task.expect("Arguments error")));
    let context = match subcommand.new_context(!cli.quiet) {
        Ok(context) => context,
        Err(e) => std::process::exit(handle_error(e, cli.quiet, "")),
    };

    let log_level = if cli.verbose { "DEBUG" } else { "OFF" };
    env_logger::Builder::from_env(Env::default().default_filter_or(log_level)).init();

    match subcommand.run(context.clone()) {
        Err(err) => {
            error!("Fatal error. Cause: {:?}", err.root_cause());
            std::process::exit(handle_error(err, cli.quiet, &context.keychain_service_name));
        }
        Ok(RunType::Completed(msg)) => {
            if !cli.quiet {
                println!("{}", msg)
            }
        }
        Ok(RunType::Cancelled) => {
            if !cli.quiet {
                println!("Cancelled")
            }
        }
    }
    std::process::exit(0);
}

fn handle_error(err: Error, is_quiet: bool, keychain_service_name: &str) -> i32 {
    // CheckVistError
    // Hacky: downcast the concrete error types
    // requiring specific handling
    match err.root_cause().downcast_ref::<CheckvistError>() {
        Some(CheckvistError::InvalidListError) => eprint_error("Couldn't find or access the list you are trying to add to.\nAre you using an invalid bookmark?", is_quiet),
        Some(CheckvistError::InvalidParentIdError) => eprint_error("Couldn't find the task you are trying to add a child task to.\nAre you using an valid bookmark?", is_quiet),
        Some(CheckvistError::TokenRefreshFailedError) => { eprint_logged_out(is_quiet);
            match creds::delete_api_token(keychain_service_name) {
                Err(err) => error!("Something went wrong deleting invalid api token: {}", err),
                _ => info!("Expired api token was deleted"),
            }
        }
        _possible_app_error => match err.downcast_ref::<AppError>() {
            Some(AppError::Reportable(msg)) => eprint_error(msg, is_quiet),
            _all_other_errors => {
                eprint_unexpected_error(err, is_quiet);
            }
        },
    }
    1
}

#[inline(always)]
fn eprint_error(message: &str, is_quiet: bool) {
    if is_quiet {
        return;
    };

    let out = ColourOutput::new(StreamKind::Stderr);
    out.append("\nError: ", Style::Error)
        .append(message, Style::Normal)
        .println()
        .expect("problem styling error text");
}

#[inline(always)]
fn eprint_logged_out(is_quiet: bool) {
    if is_quiet {
        return;
    }

    eprintln!(
        r#"
    You have been logged out of the Checkvist API.
    Please run cvcap again to log back in"#
    );
}

#[inline(always)]
fn eprint_unexpected_error(err: Error, is_quiet: bool) {
    if is_quiet {
        return;
    }

    let err_msg: String = format!(
        r#"

    If you want to report this, fill out an issue at 
    {}.

    To gather more details that might help solve the issue, 
    run the same command again with the '-v' switch, and
    copy the output into the issue.
            "#,
        "https://github.com/crispinb/cvcap/issues"
    );

    let out = ColourOutput::new(StreamKind::Stderr);
    out.append(format!("\nError: {}", err), Style::Error)
        .append(err_msg, Style::Normal)
        .println()
        .expect("problem styling error text");
}

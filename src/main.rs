use anyhow::{anyhow, Context, Error, Result};
use clap::{Parser, Subcommand};
use copypasta::{ClipboardContext, ClipboardProvider};
use cvcap::{CheckvistClient, CheckvistError, Task};
use dialoguer::{Confirm, Input, Password, Select};
use directories::ProjectDirs;
use env_logger::Env;
use keyring::Entry;
use log::{error, info};
use progress_indicator::ProgressIndicator;
use serde::{Deserialize, Serialize};
use std::fs::{self, create_dir_all, File};
use std::path::PathBuf;

mod progress_indicator;

// Logging.
// Convention: reserve trace and debug levels for libraries (eg. checkvist api)
// Levels used in executable:
// - error: any non-recoverable error (eg. inability to parse config toml: can recover by o662xkDtJuGaFa2verwriting)
// - warn: recoverable errors
// - info: transient info for debugging

const CONFIG_FILE_NAME: &str = "cvcap.toml";
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
    /// Add a new task to your default list (you'll be prompted if you don't have one yet)
    #[clap(name = "task text")]
    task_content: Option<String>,
    /// Choose a list to add a new task to (ie. other than your default list)
    #[clap(short = 'l', long)]
    choose_list: bool,
    /// Enable verbose logging. In case of trouble
    #[clap(short = 'v', long = "verbose", requires = "task text")]
    verbose: bool,
    /// Add a task from the clipboard instead of the command line
    #[clap(short = 'c', long, conflicts_with = "task text")]
    from_clipboard: bool,
    #[clap(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Check whether cvcap is logged in, and if it has a default list set
    #[clap(name = "status")]
    ShowStatus,
}

#[derive(Serialize, Deserialize, Debug)]
struct Config {
    #[serde(rename = "default_list_id")]
    list_id: u32,
    #[serde(rename = "default_list_name")]
    list_name: String,
}

fn main() {
    let cli = Cli::parse();

    let log_level = if cli.verbose { "DEBUG" } else { "OFF" };
    env_logger::Builder::from_env(Env::default().default_filter_or(log_level)).init();

    if let Err(err) = run_command(cli) {
        error!("Fatal error. Cause: {:?}", err.root_cause());
        display_error(err);
        std::process::exit(1);
    }

    std::process::exit(0);
}

fn display_error(err: Error) {
    // TODO: cf just .downcast to a specific type?? (a la zoxide)
    // (ie. why am I going all the way down the chain here?)
    match err.root_cause().downcast_ref() {
        Some(CheckvistError::TokenRefreshFailedError) => {
            eprint_logged_out();
            match delete_api_token() {
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

fn run_command(cli: Cli) -> Result<(), Error> {
    match (
        cli.task_content,
        clipboard_text_if_requested(cli.from_clipboard)?,
    ) {
        // task content from commandline or clipboard
        (Some(content), None) | (None, Some(content)) => add_task(content, cli.choose_list),

        // -c flag present, but cancelled by user
        (_, None) => Ok(()),

        // no task content
        _ => match cli.command {
            Some(Commands::ShowStatus) => {
                println!("{}", get_status());
                Ok(())
            }
            None => {
                error!(
                    "Couldn't find a command given these args: {:?}",
                    std::env::args()
                );
                Err(anyhow!(
                    "Something went wrong interpreting the command args"
                ))
            }
        },
    }
}

fn clipboard_text_if_requested(from_clipboard: bool) -> Result<Option<String>, Error> {
    let task_from_clipboard = if from_clipboard {
        let box_err_converter = |e| anyhow!("Error getting clipboard text: {:?}", e);
        let mut ctx = ClipboardContext::new().map_err(box_err_converter)?;
        let cliptext = ctx.get_contents().map_err(box_err_converter)?;
        if Confirm::new()
            .with_prompt(format!(r#"Add "{}" as a new task?"#, cliptext))
            .interact()?
        {
            Some(cliptext)
        } else {
            println!("Cancelled");
            None
        }
    } else {
        None
    };
    Ok(task_from_clipboard)
}

fn add_task(content: String, choose_list: bool) -> Result<()> {
    let client = get_api_client()?;
    let config = get_config(&client, choose_list)?;
    let task = Task {
        id: None,
        content,
        position: 1,
    };
    let add_task_msg = format!(
        r#"Adding task "{}" to list "{}""#,
        task.content, config.list_name
    );
    ProgressIndicator::new(".", &add_task_msg, "Task added", 250)
        .run(|| {
            client
                .add_task(config.list_id, &task)
                .map(|_t| ())
                .map_err(|e| Box::new(e) as _)
        })
        // TODO - RESEARCH NEEDED: cf -------> I don't understand. The Anyhow docs
        // claim a From implementation for Box<dyn stdErr + Send + Sync + 'static>,
        // which is what this map_err is a workaround for
        .map_err(|e| anyhow!(e))
        .context("Could not add task")
}

fn get_config(client: &CheckvistClient, user_chooses_new_list: bool) -> Result<Config> {
    match (get_config_from_file(), user_chooses_new_list) {
        (_, true) | (None, false) => {
            if !user_chooses_new_list {
                println!("No default list configured")
            };
            // TODO: how to eliminate dummy allocation here?
            let mut available_lists: Vec<(u32, String)> = vec![(0, String::new())];
            ProgressIndicator::new(".", "Fetching lists from Checkvist ", "", 250)
                .run(|| {
                    available_lists = client.get_lists().map(|lists| {
                        lists.into_iter().map(|list| (list.id, list.name)).collect()
                    })?;
                    Ok(())
                })
                .map_err(|e| anyhow!(e))
                .context("Could not get lists from Checkvist API")?;
            if let Some(user_config) = get_config_from_user(available_lists) {
                if Confirm::new()
                    .with_prompt(format!(
                        "Do you want to save '{}' as your default list for future task capture?",
                        user_config.list_name
                    ))
                    .interact()?
                {
                    create_new_config_file(&user_config).with_context(|| {
                        format!("Couldn't save config file to path {:?}", config_file_path())
                    })?;
                    println!("'{}' is now your default list", user_config.list_name);
                }

                Ok(user_config)
            } else {
                return Err(anyhow!("Could not collect config info from user"));
            }
        }
        (Some(file_config), false) => Ok(file_config),
    }
}

fn get_api_client() -> Result<CheckvistClient> {
    let token = get_api_token()?;
    Ok(CheckvistClient::new(
        "https://checkvist.com/".into(),
        token,
        // clippy warns about the unit argument, but I want it for the side effect
        #[allow(clippy::unit_arg)]
        |token| {
            save_api_token_to_keyring(token).unwrap_or(error!("Couldn't save token to keyring"))
        },
    ))
}

/// Gets Checkvist API token (see https://checkvist.com/auth/api#task_data)
/// - first attempts from local machine keyring
/// - if not available, then asks user for username/pw & gets from checkvist API, storing on keyring)
// Errors if we can't get token from either keyring or Checkvist API
// TODO: check/extend for MacOS
const KEYCHAIN_SERVICE_NAME: &str = "cvcap-api-token";
fn get_api_token() -> Result<String> {
    // retrieve checkvist username and password from keyring if exists
    let checkvist_api_token = match get_api_token_from_keyring() {
        Some((_username, password)) => password,
        None => {
            println!("cvcap is not logged in to Checkvist.\nPlease enter your Checkvist username and OpenAPI key\nYour OpenAPI key is available from https://checkvist.com/auth/profile");
            // get token from Checkvist API
            let token = CheckvistClient::get_token(
                "https://checkvist.com/".into(),
                Input::new().with_prompt("Checkvist Username").interact_text()?,
                Password::new().with_prompt("Checkvist OpenAPI key").with_confirmation("Confirm OpenAPI key", "Please enter your OpenAPI key (available from https://checkvist.com/auth/profile)").interact()?,
            )
            .context("Couldn't get token from Checkvist API")?;

            // Entry does not exist; create it
            save_api_token_to_keyring(&token)?;

            token
        }
    };

    Ok(checkvist_api_token)
}

fn save_api_token_to_keyring(token: &str) -> Result<(), Error> {
    let entry = Entry::new(KEYCHAIN_SERVICE_NAME, &whoami::username());
    entry
        .set_password(token)
        .context("Couldn't create keyring entry (for checkvist API token")?;

    Ok(())
}

fn get_api_token_from_keyring() -> Option<(String, String)> {
    let username = whoami::username();
    Entry::new(KEYCHAIN_SERVICE_NAME, &username)
        .get_password()
        .map(|pw| (username, pw))
        .ok()
}

fn get_status() -> String {
    let mut status_text = String::from("\n    - logged in to Checkvist: \t");
    match get_api_token_from_keyring() {
        Some((_username, _token)) => {
            status_text.push('✅');
        }
        None => {
            status_text.push('❌');
        }
    }

    status_text.push('\n');
    status_text.push_str("    - default list: \t\t");
    match get_config_from_file() {
        Some(config) => {
            status_text.push_str(&config.list_name);
        }
        None => {
            status_text.push('❌');
        }
    }
    status_text.push('\n');

    status_text
}

fn delete_api_token() -> Result<(), keyring::Error> {
    let os_username = whoami::username();
    let checkvist_api_token = Entry::new(KEYCHAIN_SERVICE_NAME, &os_username);
    checkvist_api_token.delete_password()
}

fn get_config_from_file() -> Option<Config> {
    let config_file = fs::read_to_string(config_file_path()).ok()?;
    let config = toml::from_str(&config_file).ok().or_else(|| {
        error!(
            "Failed to parse config file at path {:?}. Continuing without",
            config_file_path()
        );
        None
    })?;
    Some(config)
}

fn config_file_path() -> PathBuf {
    ProjectDirs::from("com", "not10x", "cvcap")
        .expect("OS cannot find HOME dir. Cannot proceed")
        .config_dir()
        .join(CONFIG_FILE_NAME)
}

fn get_config_from_user(lists: Vec<(u32, String)>) -> Option<Config> {
    println!("Use arrow keys (or j/k) to pick a list. Enter/Space to choose. ESC to cancel\n");

    user_choose_list(&lists).map(|list| {
        println!("You picked list '{}'", list.1);
        Config {
            list_id: list.0,
            list_name: list.1,
        }
    })
}

fn user_choose_list(lists: &[(u32, String)]) -> Option<(u32, String)> {
    let ids: Vec<&str> = lists.iter().map(|list| list.1.as_str()).collect();
    Select::new()
        .items(&ids)
        .interact_opt()
        // discard error here - nothing we can do so log & continue with None
        .map_err(|e| error!("{:?}", e))
        .ok()
        .flatten()
        // get list id and name as Ok val
        .map(|index| {
            lists
                .get(index)
                // if expect isn't safe here it's a lib (dialoguer) bug
                .expect("Internal error getting list from user")
                .to_owned()
        })
}

fn create_new_config_file(config: &Config) -> Result<()> {
    let config_path = config_file_path();
    let config_dir = config_path
        .parent()
        .expect("Couldn't construct config path");
    if !config_dir.is_dir() {
        create_dir_all(config_dir)?;
    }

    let json = toml::to_string(config)?;
    let _file = File::create(config_file_path())?;
    std::fs::write(config_file_path(), json)?;
    Ok(())
}

// #![allow(unused_imports, unused_variables)]
use anyhow::{anyhow, Context, Error, Result};
use clap::Parser;
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

static CONFIG_FILE_NAME: &str = "cvcap.toml";
const BANNER: &str = r"                           
  _   _   _   _   _  
 / \ / \ / \ / \ / \ 
( c | v | c | a | p )
 \_/ \_/ \_/ \_/ \_/ 
                              

";

#[derive(Parser, Debug)]
#[clap(version, name=BANNER, about = "A minimal cli capture tool for Checkvist (https://checkvist.com)")]
struct Cli {
    /// The task you wish to add to your default list (you'll be prompted if you don't have one yet)
    #[clap(name = "task text")]
    task_content: String,
    /// Choose a list to add a new task to (ie. other than your default list)
    #[clap(short = 'l', long)]
    choose_list: bool,
    /// Add a task from the clipboard instead of the command line
    #[clap(short = 'c', long)]
    from_clipboard: bool,
    /// Enable (very) verbose logging. In case of trouble
    #[clap(short = 'v', long = "verbose")]
    verbose: bool,
}

#[derive(Serialize, Deserialize, Debug)]
struct Config {
    #[serde(rename = "default_list_id")]
    list_id: u32,
    #[serde(rename = "default_list_name")]
    list_name: String,
    checkvist_username: Option<String>,
}

fn main() {
    // I'd rather print this after the message Cli::parse prints, but the latter
    // exits on error (eg. lacking compulsory args)
    // TODO: just emit as a new command?
    println!("{}", get_status());
    let cli = Cli::parse();

    // no log output by default. Overridden by -v flag (which sets to debug), or RUST_LOG env var
    let log_level = if cli.verbose { "DEBUG" } else { "OFF" };
    env_logger::Builder::from_env(Env::default().default_filter_or(log_level)).init();

    if let Err(err) = get_api_client().and_then(|client| {
        get_config(&client, cli.choose_list).and_then(|config| run_command(cli, config, &client))
    }) {
        error!("Fatal error. Root cause: {:?}", err.root_cause());

        match err.root_cause().downcast_ref() {
            Some(CheckvistError::TokenRefreshFailedError) => {
                eprintln!("You have been logged out of the Checkvist API.\nPlease run cvcap again to log back in");
                match delete_api_token() {
                    Err(err) => error!("Something went wrong deleting invalid api token: {}", err),
                    _ => info!("Expired api token was deleted"),
                }
            }
            // TODO: add standard error message for user, explaining how to log (-v) and where to send
            _ => eprintln!("\nError: {}", err),
        }
        std::process::exit(1);
    }

    std::process::exit(0);
}

fn run_command(cli: Cli, config: Config, client: &CheckvistClient) -> Result<(), Error> {
    if cli.from_clipboard {
        println!("Clipboard support coming soon!");
        std::process::exit(0);
    }

    let task = Task {
        id: None,
        content: cli.task_content,
        position: 1,
    };

    println!(
        r#"Adding task "{}" to list "{}""#,
        task.content, config.list_name
    );

    let mut p = ProgressIndicator::new(".", "Task added", 250);
    p.start()?;

    // start a thread that writes at time intervals
    let _returned_task = client
        .add_task(config.list_id, task)
        .context("Couldn't add task to list using Checkvist API")?;

    p.stop().map_err(|e| anyhow!(e))?;

    Ok(())
}

fn get_config(client: &CheckvistClient, user_chooses_new_list: bool) -> Result<Config> {
    match (get_config_from_file(), user_chooses_new_list) {
        (_, true) | (None, false) => {
            println!("Fetching lists from Checkvist");
            let mut p = ProgressIndicator::new(".", "", 250);
            p.start()?;
            let available_lists: Vec<(u32, String)> = client
                .get_lists()
                .map(|lists| lists.into_iter().map(|list| (list.id, list.name)).collect())
                .context("Could not get lists from Checkvist API")?;
            p.stop().map_err(|e| anyhow!(e))?;
            if let Some(user_config) = get_config_from_user(available_lists) {
                if Confirm::new()
                    .with_prompt(format!(
                        "Do you want to save '{}' as your new default list?",
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
// TAG - decision:  use OS username as key to store creds (rather than the more obvious Checkvist username)
//     - rationale: to retrieve api token without always prompting user, we need quick access to a key
//                  for the creds (keyring crate's 'username'). We can only get the checkvist
//                  username from the user. We could then store it in the config, but seems like
//                  an unnecessary step.
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
    let mut status_text = String::from("\ncvcap current status:\n");
    match get_api_token_from_keyring() {
        Some((_username, _token)) => {
            status_text.push_str("\t - logged in to Checkvist\n");
        }
        None => {
            status_text.push_str("\t - not logged in to Checkvist\n");
        }
    }
    match get_config_from_file() {
        Some(config) => {
            status_text.push_str("\t - default list: ");
            status_text.push_str(&config.list_name);
            status_text.push('\n');
        }
        None => {
            status_text.push_str("\t - no default list yet configured\n");
        }
    }

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
    println!("Pick a list (or hit ESC to cancel)");

    user_choose_list(&lists).map(|list| {
        println!("You picked list '{}'", list.1);
        Config {
            list_id: list.0,
            list_name: list.1,
            checkvist_username: None,
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

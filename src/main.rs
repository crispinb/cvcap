#![allow(unused_imports, unused_variables)]
use anyhow::{anyhow, Context, Error, Result};
use clap::Parser;
use cvcap::{Checklist, CheckvistClient, CheckvistError, Task};
use directories::ProjectDirs;
use env_logger::Env;
use keyring::{
    credential::{LinuxCredential, PlatformCredential},
    Entry,
};
use log::{debug, error, info, trace, warn};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::ffi::OsStr;
use std::fmt::Display;
use std::fs::{create_dir, File};
use std::path::PathBuf;
use std::{env, fs};

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
    /// The task you wish to add to your default list (you'll be prompted if there isn't one yet)
    #[clap(name = "task text")]
    task_content: String,
    /// Choose list to add task to (ie. other than your default list)
    #[clap(short = 'l', long)]
    choose_list: bool,
    /// Use text from clipboard instead of command line argument
    #[clap(short = 'c', long)]
    from_clipboard: bool,
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
    // no log output by default
    env_logger::Builder::from_env(Env::default().default_filter_or("OFF")).init();
    // env_logger::Builder::from_env(Env::default().default_filter_or("trace")).init();

    println!("{}", get_status());
    let cli = Cli::parse();

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
            _ => eprintln!("\nError: {}", err),
        }
        std::process::exit(1);
    }

    std::process::exit(0);
}

fn run_command(cli: Cli, config: Config, client: &CheckvistClient) -> Result<(), Error> {
    if cli.from_clipboard {
        todo!("get text from clipboard");
    }
    let task = Task {
        id: None,
        content: cli.task_content,
        position: 1,
    };

    println!(
        r#"Adding task "{}" to list "{}" ..."#,
        task.content, config.list_name
    );
    let returned_task = client
        .add_task(config.list_id, task)
        .context("Couldn't add task to list using Checkvist API")?;
    println!("Done");

    Ok(())
}

fn get_config(client: &CheckvistClient, user_chooses_new_list: bool) -> Result<Config> {
    match (get_config_from_file(), user_chooses_new_list) {
        (_, true) | (None, false) => {
            println!("Fetching lists from Checkvist ...");
            let available_lists: Vec<(u32, String)> = client
                .get_lists()
                .map(|lists| lists.into_iter().map(|list| (list.id, list.name)).collect())
                .context("Could not get lists from Checkvist API")?;

            if let Some(user_config) = get_config_from_user(available_lists) {
                if user_yn(&format!(
                    "Do you want to save {} as your new default list?",
                    user_config.list_name
                )) {
                    create_new_config_file(&user_config).context("Couldn't save config file")?;
                };
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
        |token| save_api_token_to_keyring(token).unwrap_or(error!("Couldn't save token to keyring"))
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
// TODO: check/extend for Windows & MacOS
const KEYCHAIN_SERVICE_NAME: &str = "cvcap-api-token";
fn get_api_token() -> Result<String> {
    // retrieve checkvist username and password from keyring if exists
    let checkvist_api_token = match get_api_token_from_keyring() {
        Some((_username, password)) => password,
        None => {
            // get token from Checkvist API
            let token = CheckvistClient::get_token(
                "https://checkvist.com/".into(),
                get_user_input("Username:", "Please enter your username <add details>")?,
                get_user_input("remote key TBD more info", "do it")?,
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
        Some((username, token)) => {
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
    println!("Your lists:\n");
    for (i, list) in lists.iter().enumerate() {
        println!("{}: {}", i + 1, list.1);
    }
    println!("\n");

    let chosen_list = loop {
        let chosen_index: usize = get_user_input(
            &format!(
                "\nSelect a list by entering a number between 1 and  {}\n",
                lists.len()
            ),
            "",
        )
        .unwrap();
        if let Some(list) = lists.get(chosen_index - 1) {
            break list;
        }
    };

    Some(Config {
        list_id: chosen_list.0,
        list_name: chosen_list.1.clone(),
        checkvist_username: None,
    })
}

fn create_new_config_file(config: &Config) -> Result<()> {
    let config_path = config_file_path();
    let config_dir = config_path
        .parent()
        .expect("Couldn't construct config path");
    if !config_dir.is_dir() {
        create_dir(config_dir)?;
    }

    let json = toml::to_string(config)?;
    let file = File::create(config_file_path())?;
    std::fs::write(config_file_path(), json)?;
    Ok(())
}

/// Get user input, returning any type that can be converted
/// from a string.
/// Cycles supplied prompts until input is successful
/// or an error is returned
// TODO - RESEARCH NEEDED:
//      - add a validator (closure presumably but on quick attempt I got in a mess with types)
fn get_user_input<T: std::str::FromStr>(prompt: &str, correction: &str) -> Result<T> {
    let user_input = loop {
        println!("{}", prompt);
        let mut buf = String::new();
        std::io::stdin().read_line(&mut buf).map_err(|err| {
            warn!("Couldn't get user input from stdin");
            err
        })?;

        match buf.trim().parse() {
            Ok(i) => break i,
            Err(_) => {
                println!("{}", correction);
                continue;
            }
        };
    };

    Ok(user_input)
}

fn user_yn(yes_no_question: &str) -> bool {
    println!("{} [Y/N]?", yes_no_question);

    loop {
        let mut buf = String::new();
        std::io::stdin()
            .read_line(&mut buf)
            .expect("Something went badly wrong");
        let user_input = buf.trim().to_lowercase();
        match user_input.as_str() {
            "y" => break true,
            "n" => break false,
            _ => {
                println!("Please answer Y/y or N/n");
                continue;
            }
        }
    }
}

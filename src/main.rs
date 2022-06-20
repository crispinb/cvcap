#![allow(unused_imports, unused_variables)]
use anyhow::{anyhow, Context, Error, Result};
use clap::{Args, Command, Parser, Subcommand};
use cvcap::{Checklist, CheckvistClient, CheckvistError, Task};
use directories::ProjectDirs;
use log::{debug, error, info, trace, warn};
use serde::{Deserialize, Serialize};
use std::ffi::OsStr;
use std::fmt::Display;
use std::fs::{create_dir, File};
use std::path::PathBuf;
use std::{env, fs};

// Logging.
// Convention: reserve trace and debug levels for called crates (eg. checkvist api)
// Levels used in executable:
// - error: any recoverable error (eg. inability to parse config toml: can recover by overwriting)
// - warn: non-error potential problems
// - info: transient info for debugging

static CONFIG_FILE_NAME: &str = "cvcap.toml";
// TODO: get/create version during build
static VERSION: &str = "0.1";
const BANNER: &str = r"                           
  _   _   _   _   _  
 / \ / \ / \ / \ / \ 
( c | v | c | a | p )
 \_/ \_/ \_/ \_/ \_/ 
                              

";

#[derive(Parser, Debug)]
#[clap(name = BANNER)]
#[clap(about = "A minimal Checkvist (https://checkvist.com) capture tool ")]
#[clap(version = VERSION)]
struct Cli {
    /// The task you wish to add
    #[clap(name="task")]
    task_content: String,
    /// Add task to a different list (ie. other than your default list)
    #[clap(short='l', long)]
    pick_list: bool,
    /// Add task from text on clipboard
    #[clap(short='c', long)]
    from_clipboard: bool,
}

#[derive(Serialize, Deserialize, Debug)]
struct Config {
    default_list_id: u32,
    default_list_name: String,
}

fn main() -> Result<()> {
    env_logger::init();
    let cli = Cli::parse();
    let token = api_token()?;
    let client = CheckvistClient::new("https://checkvist.com/".into(), token);

    if cli.pick_list {
        todo!("so you want a new list?");
    }
    if cli.from_clipboard {
        todo!("get text from clipboard");
    }

    let config = if let Some(config) = get_config_from_file() {
        config
    } else {
        let available_lists: Vec<(u32, String)> = client
            .get_lists()
            .map(|lists| lists.into_iter().map(|list| (list.id, list.name)).collect())
            .context("Could not retrieve lists from Checkvist API")?;

        if let Some(config) = get_config_from_user(available_lists) {
            if user_yn("Do you want to add your list as the default to a new config file?") {
                create_new_config_file(&config).context("Couldn't create config file")?;
            };
            config
        } else {
            return Err(anyhow!("Could not collect config info from user"));
        }
    };

    let task = Task {
        id: None,
        content: cli.task_content,
        position: 1,
    };

    let returned_task = client
        .add_task(config.default_list_id, task)
        .context("Couldn't add task to list using Checkvist API")?;
    println!(
        r#"Added task "{}" to list "{}""#,
        returned_task.content, config.default_list_name
    );

    Ok(())
}

// TODO - RESEARCH NEEDED:
//        - how to capture and where to store token
const TOKEN_KEY: &str = "CHECKVIST_API_TOKEN";
fn api_token() -> Result<String> {
    let key: &OsStr = OsStr::new(TOKEN_KEY);
    let need_token_msg: String = format!("you must set the {:?} environment variable", key);
    let token = env::var(key).context(need_token_msg)?;
    Ok(token)
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

fn get_config_from_user(lists: Vec<(u32, String)>) -> Option<Config> {
    println!("Your lists:\n");
    for (i, list) in lists.iter().enumerate() {
        println!("{}: {}", i + 1, list.1);
    }
    println!("\n");

    let chosen_list = loop {
        println!(
            "\nSelect a list by entering a number between 1 and  {}\n",
            lists.len()
        );
        let mut buf = String::new();
        std::io::stdin().read_line(&mut buf).ok().or_else(|| {
            warn!("Couldn't get user input for unknown reason");
            None
        })?;
        let chosen_index: usize = match buf.trim().parse() {
            Ok(i) => i,
            Err(_) => {
                continue;
            }
        };
        if let Some(list) = lists.get(chosen_index - 1) {
            break list;
        }
    };

    Some(Config {
        default_list_id: chosen_list.0,
        default_list_name: chosen_list.1.clone(),
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

#![allow(unused_imports, unused_variables)]
use clap::{Args, Command, Parser, Subcommand};
use cvcap::{CheckvistClient, CheckvistError, Task};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fmt::Display;
use std::fs::{create_dir, File};
use std::path::PathBuf;
use std::{env, fs};

static CONFIG_FILE_NAME: &str = "cvcap.toml";
// TODO: ge&t this during build
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
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    List {
        // name: String,
        list_id: i16,
    },
    Add {
        content: String,
    },
}

#[derive(Debug)]

// TODO - RESEARCH NEEDED:
//        Variants per external error type are ridiculous.
//        What I want is a type with a Box dyn inner error, but
//        this always results in compile errors for Display ( to do
// with displaying an unsized box value)
enum CliError {
    Error { message: String },
    IOError(std::io::Error),
    TomlDeserialisationError(toml::de::Error),
    TomlSerialisationError(toml::ser::Error),
    CheckvistError(CheckvistError),
}

impl Display for CliError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CliError::Error { message } => write!(f, "{}", message),
            // TODO: fix this when I have a solution to the inner error issue
            _ => write!(f, "TBD"),
        }
    }
}

impl Error for CliError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            CliError::Error { message: _message } => None,
            CliError::IOError(err) => Some(err),
            CliError::TomlDeserialisationError(err) => Some(err),
            CliError::TomlSerialisationError(err) => Some(err),
            CliError::CheckvistError(err) => Some(err),
        }
    }
}

impl From<std::io::Error> for CliError {
    fn from(err: std::io::Error) -> Self {
        CliError::IOError(err)
    }
}

impl From<toml::de::Error> for CliError {
    fn from(err: toml::de::Error) -> Self {
        CliError::TomlDeserialisationError(err)
    }
}

impl From<toml::ser::Error> for CliError {
    fn from(err: toml::ser::Error) -> Self {
        CliError::TomlSerialisationError(err)
    }
}

impl From<CheckvistError> for CliError {
    fn from(err: CheckvistError) -> Self {
        CliError::CheckvistError(err)
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct Config {
    default_list_id: u32,
    default_list_name: String,
}

fn main() -> Result<(), CliError> {
    let cli = Cli::parse();

    // TODO - RESEARCH NEEDED:
    //        - how to capture and where to store token
    // TODO: switch to OsString / &OSStr
    const TOKEN_KEY: &str = "CHECKVIST_API_TOKEN";
    let need_token_msg: String = format!("you must set the {} environment variable", TOKEN_KEY);
    let token = match env::var(TOKEN_KEY) {
        Ok(token) => token,
        Err(err) => {
            return Err(CliError::Error {
                message: "Can't get token from environment".into(),
            })
        }
    };

    let client = CheckvistClient::new("https://checkvist.com/".into(), token);

    match cli.command {
        Commands::List { list_id } => todo!(),
        Commands::Add { content } => {
            // TODO - RESEARCH NEEDED: 
            //        error handling
            let config = match get_config_from_file() {
                Ok(config) => config,
                Err(err) => {
                    let config = get_config_from_user(&client).unwrap();
                    if user_yn("Do you want to add your list as the default to a new config file?") {
                        create_new_config_file(&config).unwrap();
                    }
                    config
                }
            };

            // FIXME: randomish weirdness. Fails all sending to (and only to) @main, but  works in curl --header "X-Client-Token: $CHECKVIST_API_TOKEN" --json '{"content": "curl add", "position": 1}'  "https://checkvist.com/checklists/565368/tasks.json"
            // weird, so see if there's anything to learn from the fix, and add test
            // - "learn german"

            let task = Task {
                // not sure what to do about ids in new (local only) tasks
                id: config.default_list_id,
                content,
                position: 1,
            };

            let success_message = match client.add_task(config.default_list_id, task) {
                Ok(returned_task) => returned_task.content,
                Err(err) => return Err(CliError::CheckvistError(err)),
            };

            println!(
                r#"Added task "{}" to list "{}""#,
                success_message, config.default_list_name
            );
        }
    }

    Ok(())
}

fn get_config_from_file() -> Result<Config, CliError> {
    let config_file = fs::read_to_string(config_file_path())?;
    let config = toml::from_str(&config_file)?;
    Ok(config)
}

fn config_file_path() -> PathBuf {
    ProjectDirs::from("com", "not10x", "cvcap")
        .expect("OS cannot find HOME dir. Cannot proceed")
        .config_dir()
        .join(CONFIG_FILE_NAME)
}

// TODO - REFACTOR: merge with get_config_from_user?
fn user_yn(yes_no_question: &str) -> bool {
    println!("{} [Y/N]?", yes_no_question);

    loop {
        let mut buf = String::new();
        std::io::stdin()
            .read_line(&mut buf)
            .expect("Something went badly wrong");
        let temp_remove = buf.trim().to_lowercase();
        // TODO - RESEARCH NEEDED:
        //        why are t1 and t2 different types here?
        let t1 = temp_remove.as_str();
        let t2 = &temp_remove;
        // match t2 {
        match t1 {
            "y" => break true,
            "n" => break false,
            _ => {
                println!("Please answer Y/y or N/n");
                continue;
            }
        }
    }
}

fn get_config_from_user(client: &CheckvistClient) -> Result<Config, CliError> {
    let available_lists: Vec<(u32, String)> = client
        .get_lists()
        .map(|lists| lists.into_iter().map(|list| (list.id, list.name)).collect())?;

    println!("Your lists:\n");
    for (i, list) in available_lists.iter().enumerate() {
        println!("{}: {}", i + 1, list.1);
    }
    println!("\n");

    // TODO - RESEARCH NEEDED:
    //        idiomatic way of collecting cmdline input
    let chosen_list = loop {
        println!(
            "\nSelect a list by entering a number between 1 and  {}\n",
            available_lists.len()
        );
        let mut buf = String::new();
        std::io::stdin().read_line(&mut buf)?;
        let chosen_index: usize = match buf.trim().parse() {
            Ok(i) => i,
            Err(_) => {
                continue;
            }
        };
        if let Some(list) = available_lists.get(chosen_index - 1) {
            break list;
        }
    };

    Ok(Config {
        default_list_id: chosen_list.0,
        default_list_name: chosen_list.1.clone(),
    })
}

fn create_new_config_file(config: &Config) -> Result<(), CliError> {
    let config_path = config_file_path();
    let config_dir = config_path.parent().ok_or(CliError::Error {
        message: "Can't find a config directory".into(),
    })?;
    if !config_dir.is_dir() {
        create_dir(config_dir)?;
    }

    let json = toml::to_string(config)?;
    let file = File::create(config_file_path())?;
    std::fs::write(config_file_path(), json)?;
    Ok(())
}

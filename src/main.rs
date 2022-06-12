#![allow(unused_imports, unused_variables)]
use clap::{Args, Command, Parser, Subcommand};
use cvcap::{CheckvistClient, CheckvistError, Task};
use directories::ProjectDirs;
use serde::Deserialize;
use std::error::Error;
use std::fmt::Display;
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
struct CliError {
    message: String,
    inner_error: Box<dyn Error>, // see impl Error
}

impl Display for CliError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl Error for CliError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&*self.inner_error)
    }
}

impl From<std::io::Error> for CliError {
    fn from(err: std::io::Error) -> Self {
        CliError {
            message: "IO Error".into(),
            inner_error: Box::new(err),
        }
    }
}

impl From<toml::de::Error> for CliError {
    fn from(err: toml::de::Error) -> Self {
        CliError {
            message: "Bad Toml in config file".into(),
            inner_error: Box::new(err),
        }
    }
}

impl From<CheckvistError> for CliError {
    fn from(err: CheckvistError) -> Self {
        CliError {
            message: "Error calling Checkvist API".into(),
            inner_error: Box::new(err),
        }
    }
}

#[derive(Deserialize, Debug)]
struct Config {
    default_list_id: u32,
    default_list_name: String,
}

fn main() -> Result<(), CliError> {
    let cli = Cli::parse();

    // TODO - RESEARCH NEEDED:
    //        - how to capture and where to store token
    const TOKEN_KEY: &str = "CHECKVIST_API_TOKEN";
    let need_token_msg: String = format!("you must set the {} environment variable", TOKEN_KEY);
    let token = match env::var(TOKEN_KEY) {
        Ok(token) => token,
        Err(err) => {
            return Err(CliError {
                message: need_token_msg,
                inner_error: Box::new(err),
            })
        }
    };

    let client = CheckvistClient::new("https://checkvist.com/".into(), token);

    match cli.command {
        Commands::List { list_id } => {}

        Commands::Add { content } => {
            let config = match get_config_from_file() {
                Ok(config) => config,
                // TODO: make distinction between no file VS badly formattted toml?
                // TODO: handle error here or in func?
                Err(err) => get_config_from_user(&client).unwrap(),
            };

            // FIXME: single-case weirdness. Fails all sending to (and only to) @main, but  works in curl --header "X-Client-Token: $CHECKVIST_API_TOKEN" --json '{"content": "curl add", "position": 1}'  "https://checkvist.com/checklists/565368/tasks.json"
            // weird, so see if there's anything to learn from the fix
            // - "learn german"

            let task = Task {
                // not sure what to do about ids in new (local only) tasks
                id: config.default_list_id,
                content,
                position: 1,
            };

            let success_message = match client.add_task(config.default_list_id, task) {
                Ok(returned_task) => returned_task.content,
                Err(err) => {
                    return Err(CliError {
                        message: err.to_string(),
                        inner_error: Box::new(err),
                    })
                }
            };

            println!(
                r#"Added task "{}" to list "{}""#,
                success_message, config.default_list_name
            );
        }
    }

    Ok(())
}

// TODO: add creation on first run
fn get_config_from_file() -> Result<Config, CliError> {
    let config_path = ProjectDirs::from("com", "not10x", "cvcap")
        .expect("OS cannot find HOME dir. Cannot proceed")
        .config_dir()
        .join(CONFIG_FILE_NAME);
    let config_file = fs::read_to_string(config_path)?;
    let config = toml::from_str(&config_file)?;
    Ok(config)
}

fn get_config_from_user(client: &CheckvistClient) -> Result<Config, CliError> {
    let available_lists: Vec<(u32, String)> = client
        .get_lists()
        .map(|lists| lists.into_iter().map(|list| (list.id, list.name)).collect())?;

    println!("Your lists:\n");
    let mut i = 0;
    for list in &available_lists {
        i += 1;
        println!("{}: {}", i, list.1);
    }
    println!("\n");

    // TODO - RESEARCH NEEDED:
    //        idiomatic way of collecting cmdline input
    let chosen_list = loop {
        println!(
            "\nPlease select a list by entering a number between 1 and  {}\n",
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

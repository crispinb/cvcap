#![allow(unused_imports, unused_variables)]
use checkvistcli::{CheckvistClient, Task};
use clap::{Args, Command, Parser, Subcommand};
use directories::ProjectDirs;
use serde::Deserialize;
use std::error::Error;
use std::fmt::Display;
use std::{env, fs};

static CONFIG_FILE_NAME: &str = "cvcap.toml";
// TODO: ge&t this during build
static VERSION: &str = "0.1";
const BANNER: &str = r"                           
                           
                          
 __        __   __,    _  
/    |  |_/    /  |  |/ \_
\___/ \/  \___/\_/|_/|__/ 
                    /|    
                    \|    
                              
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

#[derive(Deserialize, Debug)]
struct Config {
    default_list_id: u32,
    default_list_name: String,
}

fn main() -> Result<(), CliError> {

    let config = match read_or_create_config() {
        Ok(config) => config,
        Err(err) => todo!("figure out {}", err),
    };

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
        Commands::List { list_id } => todo!("TBD"),
        Commands::Add { content } => {
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
fn read_or_create_config() -> Result<Config, CliError> {
    let config_path = ProjectDirs::from("com", "not10x", "cvcap")
        .expect("OS cannot find HOME dir. Cannot proceed")
        .config_dir()
        .join(CONFIG_FILE_NAME);
    let config_file = fs::read_to_string(config_path)?;
    let config = toml::from_str(&config_file)?;
    Ok(config)
}

#![allow(unused_imports, unused_variables)]
use checkvistcli::{CheckvistClient, Task};
use clap::{Args, Command, Parser, Subcommand};
use std::env;
use std::error::Error;
use std::fmt::Display;

// TODO: ge&t this during build
static VERSION: &str = "0.1";

const BANNER: &str = r"  ____ _               _           _     _             _ _
/ ___| |__   ___  ___| | ____   _(_)___| |_       ___| (_)
| |   | '_ \ / _ \/ __| |/ /\ \ / / / __| __|____ / __| | |
| |___| | | |  __/ (__|   <  \ V /| \__ \ ||_____| (__| | |
\____|_| |_|\___|\___|_|\_\  \_/ |_|___/\__|     \___|_|_|



";

#[derive(Parser)]
#[clap(version = VERSION)]
#[clap(name = BANNER)]
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

fn main() -> Result<(), CliError> {
    // TODO: get from config file and/or first run
    let list_name = "Dev List";
    let list_id = 774394;
    //

    let cli = Cli::parse();

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
        Commands::List { list_id } => todo!(),
        Commands::Add { content } => {
            // add task
            let task = Task {
                id: 1,
                content,
                position: 1,
            };

            let success_message = match client.add_task(list_id, task) {
                Ok(returned_task) => returned_task.content,
                Err(err) => {
                    return Err(CliError {
                        message: err.to_string(),
                        inner_error: Box::new(err)
                    })
                }
            };

            println!(
                r#"Added task "{}" to list "{}""#,
                success_message, list_name
            );
        }
    }

    Ok(())
}

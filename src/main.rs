#![allow(unused_imports, unused_variables)]
use checkvistcli::{CheckvistClient, Task};
use clap::Parser;
use clap::{Arg, Command};
use std::env;
use std::error::Error;
use std::fmt::Display;

// TODO: ge&t this from build system in some way?
// static VERSION: &str = "0.1";

// TODO - RESEARCH NEEDED: 
//        how to get clap parse to deal with this in the about
    // const BANNER: &str = r"  ____ _               _           _     _             _ _ 
// / ___| |__   ___  ___| | ____   _(_)___| |_       ___| (_)
// | |   | '_ \ / _ \/ __| |/ /\ \ / / / __| __|____ / __| | |
// | |___| | | |  __/ (__|   <  \ V /| \__ \ ||_____| (__| | |
// \____|_| |_|\___|\___|_|\_\  \_/ |_|___/\__|     \___|_|_|
                                                          
// ";

// #[clap(long_about=format!("{}A Checkvist (https://checkvist.com) command line interface,\nfocused on quick capture of data to Checkvist lists", &BANNER))]

#[derive(Parser)]
#[clap(author="Crispin Bennett", version="0.1", about="a thing")]
#[clap(about="about", long_about="an about string")]
#[clap(name = "Checkvist Cli this is the name")]
struct Cli {
    content: String,
}

#[derive(Debug)]
struct CliError {
    message: String,
    // innerError: Box<dyn Error>, // see impl Error
}

impl Display for CliError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Display: {}", self.message)
    }
}

// TODO - RESEARCH NEEDED: 
//        types needed for this to compile
// impl Error for CliError {
//     fn cause(&self) -> Option<&dyn Error> {
//         Some(&self.innerError)
//     }
// }

fn main() -> Result<(), CliError> {
    // TODO: get from config file and/or first run
    let list_name = "Dev List";
    let list_id = 774394;
    //

    const TOKEN_KEY: &str = "CHECKVIST_API_TOKEN";
    let need_token_msg: String = format!("you must set the {} environment variable", TOKEN_KEY);
    let token = match env::var(TOKEN_KEY) {
        Ok(token) => token,
        Err(_) => return Err(CliError{message: need_token_msg}),
    };


    let cli = Cli::parse();
    let client = CheckvistClient::new("https://checkvist.com/".into(), token);
    
    // TODO - RESEARCH NEEDED: 
    //        Multiple commands 
    // ACTIONS
    // get list
    // let list = client.get_list(774394).unwrap();
    // println!("list details: {:?}", list);

    // add task
    let task = Task {
        id: 1,
        content: cli.content,
        position: 1,
    };

    let success_message = match client.add_task(list_id, task) {
        Ok(returned_task) => returned_task.content,
        Err(err) => {
            return Err(CliError {
                message: err.to_string(),
            })
        }
    };

    println!(r#"Added task "{}" to list "{}""#, success_message, list_name);

    Ok(())
}

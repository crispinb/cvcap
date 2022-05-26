#![allow(unused_imports)]
use std::env;

use checkvistcli::{ChecklistClient, Task, TempTaskForAdding};
use clap::{Arg, Command};

// TODO: ge&t this from build system in some way?
static VERSION: &str = "0.1";

#[tokio::main]
async fn main() {
    // TODO: conditional use of canned data with wiremock somehow?
let banner =r"  ____ _               _           _     _             _ _ 
/ ___| |__   ___  ___| | ____   _(_)___| |_       ___| (_)
| |   | '_ \ / _ \/ __| |/ /\ \ / / / __| __|____ / __| | |
| |___| | | |  __/ (__|   <  \ V /| \__ \ ||_____| (__| | |
\____|_| |_|\___|\___|_|\_\  \_/ |_|___/\__|     \___|_|_|
                                                          
";
    let matches = Command::new("checkvistcli")
        .version(VERSION)
        // .about("fark!")
        .about(banner)
        // .about(format!("{}A Checkvist (https://checkvist.com) command line interface,\nfocused on quick capture of data to Checkvist lists", &banner))
        .author("Crispin Bennett")
        // .arg(Arg::new("command"))
        .arg(Arg::new("content")
             .short('t')
        .value_name("content"))
        .after_help("This is the after help section")
        .get_matches();
        
        let task_content = matches.value_of("content").unwrap();

    const TOKEN_KEY: &str = "CHECKVIST_API_TOKEN";
    let need_token_msg: String = format!("you must set the {} environment variable", TOKEN_KEY);
    let token = env::var(TOKEN_KEY).expect(&need_token_msg);

    // hardcoded attempt
    let client = ChecklistClient::new(
        "https://checkvist.com/".into(),
        token
    );
    let task = TempTaskForAdding {
        content: task_content.into(),
        position: 1,
    };
    let added_task = client.add_task(774394, &task).await.unwrap();
    println!("added task {:?}", added_task);
    // let client = ChecklistClient::new(
    //     "https://checkvist.com/".into(),
    //     "bad_token".into(),
    //     // "HRpvPJqF4uvwVR8jQ3mkiqlwCm7Y6n".into(),
    // );
    // let list = client.get_list(774394).await.unwrap();
    // println!("list details: {:?}", list);

    // let tasks = client.get_all_tasks(774394).await.unwrap();
    // println!("tasks: {:?}", tasks);
}

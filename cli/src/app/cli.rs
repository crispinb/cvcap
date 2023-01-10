use anyhow::Result;
use clap::{Parser, Subcommand};

use super::action;
use super::context::Context;

const BANNER: &str = r"                           
  _   _   _   _   _  
 / \ / \ / \ / \ / \ 
( c | v | c | a | p )
 \_/ \_/ \_/ \_/ \_/ 
                              

";

#[derive(Parser, Debug)]
#[clap(version, name=BANNER, about = "A minimal cli capture tool for Checkvist (https://checkvist.com)")]
#[clap(arg_required_else_help = true, subcommand_negates_reqs = true)]
pub struct Cli {
    /// The task content to capture
    #[clap(name = "task content", value_name = "TASK")]
    pub task: Option<String>,
    #[clap(subcommand)]
    pub subcommand: Option<Command>,
    /// Enable verbose logging. In case of trouble
    #[clap(short = 'v', long = "verbose", global = true)]
    pub verbose: bool,
    /// Reduces output, and requires no interaction
    #[clap(short = 'q', long = "quiet", global = true, conflicts_with = "verbose")]
    pub quiet: bool,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Capture a task from commandline, clipboard, or stdin
    Add(action::Add),
    /// Check whether cvcap is logged in, and if it has a default list set
    #[clap(name = "status")]
    ShowStatus(action::ShowStatus),
    /// Removes all login data for the logged in user
    #[clap(name = "logout")]
    LogOut(action::LogOut),
}

impl Command {
    // Create a default add command, with a content string and no options.
    // Can't  use std::default as we need the arg
    pub fn default(task_content: &str) -> Self {
        Self::Add(action::Add::new(task_content))
    }
}

impl action::Action for Command {
    fn run(self, context: Context) -> Result<action::RunType> {
        match self {
            Command::Add(add) => add.run(context),
            Command::ShowStatus(cmd) => cmd.run(context),
            Command::LogOut(cmd) => cmd.run(context),
        }
    }
}

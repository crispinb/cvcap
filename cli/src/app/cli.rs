use anyhow::Result;
// TODO: narrow
use bpaf::*;
// use bpaf::{long, short, command, construct, parsers};

use super::action;
use super::context::Context;

const VERSION: &str = env!("CARGO_PKG_VERSION");
const BANNER: &str = r"                           
  _   _   _   _   _  
 / \ / \ / \ / \ / \ 
( c | v | c | a | p )
 \_/ \_/ \_/ \_/ \_/ 
                              
A minimal cli capture tool for Checkvist (https://checkvist.com)
";

#[derive(Debug)]
pub struct Cli {
    pub interactivity_level: InteractivityLevel,
    pub subcommand: Command,
}

/// Silent - no messages displayed, and no input prompted
/// Normal - messages displayed intended to be useful for regular
///          interactive use; input will be prompted where required
/// Verbose - as per Normal, with added logging for troubleshooting
///         purposes
#[derive(Debug, Clone, PartialEq)]
pub enum InteractivityLevel {
    Silent,
    Normal,
    Verbose,
}

#[derive(Debug, Clone)]
pub enum Command {
    Add(action::AddTask),
    ShowStatus(action::ShowStatus),
    LogOut(action::LogOut),
    AddBookmark(action::AddBookmark),
    ShowUsage,
}

impl Cli {
    pub fn parse() -> Self {
        let verbose = long("verbose")
            .short('v')
            .help("Enable verbose logging. In case of trouble")
            .req_flag(InteractivityLevel::Verbose);
        let quiet = long("quiet")
            .short('q')
            .help("Reduces output, and requires no interaction")
            .req_flag(InteractivityLevel::Silent);
        let interactivity_level = construct!([verbose, quiet]).fallback(InteractivityLevel::Normal);

        let build_show_usage = || {
            let show_usage = pure(Command::ShowUsage);
            let s = construct!(show_usage).to_options();
            command("help", s)
        };

        let add_task_command = action::AddTask::command();
        let logout_command = action::LogOut::command();
        let status_command = action::ShowStatus::command();
        let add_bookmark_command = action::AddBookmark::command();

        let subcommand = construct!([
            logout_command,
            status_command,
            add_bookmark_command,
            add_task_command,
            build_show_usage(),
        ])
        .fallback(Command::ShowUsage);

        let cli = construct!(Cli {
            interactivity_level,
            subcommand,
        })
        .to_options()
        .descr(BANNER)
        .version(VERSION);

        cli.run()
    }
}

impl Command {
    /// Allow subcommands to tailor the context
    pub fn new_context(&self, allow_interaction: bool) -> Result<Context> {
        match self {
            Command::Add(_) => Context::new(allow_interaction),
            Command::ShowStatus(_) => Context::new(false),
            Command::LogOut(_) => Context::new(false),
            Command::AddBookmark(_) => Context::new(allow_interaction),
            Command::ShowUsage => Context::new(allow_interaction),
        }
    }
}

impl action::Action for Command {
    fn run(self, context: Context) -> Result<action::RunType> {
        match self {
            Command::Add(add) => add.run(context),
            Command::ShowStatus(cmd) => cmd.run(context),
            Command::LogOut(cmd) => cmd.run(context),
            Command::AddBookmark(cmd) => cmd.run(context),
            // TODO: get bpaf's usage message 
            Command::ShowUsage => Ok(crate::RunType::Completed("USAGE MESSAGE TBD".into())),
        }
    }
}

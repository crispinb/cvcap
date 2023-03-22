use std::io::{self, Read};

use anyhow::{anyhow, Context as ErrContext, Result as AnyhowResult};
use bpaf::positional;
use bpaf::{command, construct, long, parsers::ParseCommand, Parser};
use dialoguer::Confirm;

use super::{Action, RunType};
use crate::app::{cli, context, interaction};
use crate::clipboard;
use crate::colour_output::{ColourOutput, StreamKind, Style};
use crate::context::ConfigAbsentError;
use crate::progress_indicator::ProgressIndicator;
use cvapi::{CheckvistClient, Task};

type Result<T> = std::result::Result<T, AddTaskError>;

#[derive(Clone, Debug)]
enum ContentSource {
    Stdin,
    Clipboard,
    Arg(String),
}

#[derive(Clone, Debug)]
enum DestinationSource {
    Config,
    //TODO: how to make this muttually excusive with `-q`? A guard?
    PromptUser,
    Bookmark(String),
}

#[derive(Debug, Clone)]
pub struct AddTask {
    content_source: ContentSource,
    destination_source: DestinationSource,
}

impl Action for AddTask {
    fn run(self, context: context::Context) -> AnyhowResult<RunType> {
        match self.create_job(&context) {
            Ok(job) => job.run(context),
            Err(AddTaskError::UserCancellation) => Ok(RunType::Cancelled),
            Err(AddTaskError::Unhandled(e)) => Err(e),
        }
    }
}

impl AddTask {
    pub fn command() -> ParseCommand<cli::Command> {
        let from_clipboard = long("clipboard")
            .short('c')
            .help("Add a task from the clipboard instead of the command line")
            .req_flag(ContentSource::Clipboard);
        let from_stdin = long("stdin")
            .short('s')
            .help("Add a task from stdin (ie. piped in) instead of the command line")
            .req_flag(ContentSource::Stdin);
        let content_arg = positional("TASK_CONTENT");
        let from_content_arg = construct!(ContentSource::Arg(content_arg));
        let content_source = construct!([from_clipboard, from_stdin, from_content_arg]);

        let to_user_prompted = long("choose_list")
            .short('l')
            .help("Choose from all available Checkvist lists")
            .req_flag(DestinationSource::PromptUser);
            // .guard(|_| false, "thats no good!");

        let bookmark_arg = long("bookmark")
            .short('b')
            .help("Add task to a bookmarked location")
            .argument::<String>("BOOKMARK");
        let to_bookmark = construct!(DestinationSource::Bookmark(bookmark_arg));
        let destination_source =
            construct!([to_user_prompted, to_bookmark]).fallback(DestinationSource::Config);

        let add_task = construct!(AddTask {
            // destination_source must precede content_source
            // as latter has a positional
            destination_source,
            content_source,
        });

        let add_task_cli_command = construct!(cli::Command::Add(add_task))
            .to_options()
            .descr("Capture a task from commandline, clipboard, or stdin");

        command("add", add_task_cli_command)
    }

    fn create_job(self, context: &context::Context) -> Result<AddTaskJob> {
        let client = context.api_client()?;
        let config = context.config.as_ref()?;
        let (list_id, location_name, parent_id, possible_new_default_list) =
            match self.destination_source {
                DestinationSource::Config => {
                    (config.list_id, config.list_name.clone(), None, false)
                }
                DestinationSource::PromptUser => {
                    let msg = ColourOutput::new(StreamKind::Stdout)
                        .append("Current default list: ", Style::Normal)
                        .append(&config.list_name, Style::ListName);
                    let Some(user_selected_list) = interaction::user_select_list(&client, msg)? else {
                        return Err(AddTaskError::UserCancellation);
                    };
                    (
                        user_selected_list.0,
                        user_selected_list.1,
                        None,
                        config.list_id != user_selected_list.0,
                    )
                }
                DestinationSource::Bookmark(ref bookmark_name) => {
                    match config.bookmark(bookmark_name) {
                Some(bookmark) => (bookmark.location.list_id, bookmark.name, bookmark.location.parent_task_id, false),
                None => return Err(AddTaskError::Unhandled(anyhow!("You tried to add a task to the bookmark '{}', but no bookmark of that name was found", bookmark_name))),
                            }
                }
            };

        let content = match self.content_source {
            ContentSource::Arg(content) => Ok(content),
            ContentSource::Stdin => self.get_content_from_stdin(),
            ContentSource::Clipboard => self.get_content_from_clipboard(context.allow_interaction),
        }?;

        let task = Task {
            id: None,
            parent_id,
            content,
            position: 1,
        };

        Ok(AddTaskJob {
            client,
            task,
            list_id,
            location_name,
            possible_new_default_list,
        })
    }

    fn get_content_from_stdin(&self) -> Result<String> {
        if !is_content_piped() {
            return Err(AddTaskError::Unhandled(anyhow!(
                "Tried to read from stdin pipe, but nothing was piped"
            )));
        }
        let mut buffer = String::new();

        io::stdin().lock().read_to_string(&mut buffer)?;
        Ok(buffer)
    }

    fn get_content_from_clipboard(&self, allow_interaction: bool) -> Result<String> {
        let Some(cliptext) = clipboard::get_clipboard_as_string() else {
            return Err(AddTaskError::Unhandled(anyhow!("Couldn't get clipboard contents")));
        };
        if allow_interaction
            || Confirm::new()
                .with_prompt(format!(r#"Add "{}" as a new task?"#, cliptext))
                .interact()?
        {
            Ok(cliptext)
        } else {
            Err(AddTaskError::UserCancellation)
        }
    }
}

// check out this thread for raw vs validated data naming: https://elk.zone/fosstodon.org/@rauschma/109904332263316273
// I've decided here on Command vs Job which suits the domain
struct AddTaskJob {
    client: CheckvistClient,
    task: Task,
    list_id: u32,
    /// bookmark or list name
    location_name: String,
    possible_new_default_list: bool,
}

impl AddTaskJob {
    fn run(self, context: context::Context) -> AnyhowResult<RunType> {
        if context.allow_interaction {
            let msg = self.user_message();
            let new_name = self.location_name.clone();
            let new_list_id = self.list_id;
            let user_msg = || msg.println().expect("Problem printing colour output");
            let do_job = || self.add_task();
            let config_dirty = ProgressIndicator::new('.', Box::new(user_msg), 250).run(do_job)?;
            if config_dirty {
                let new_config = crate::config::Config {
                    list_id: new_list_id,
                    list_name: new_name,
                    ..context.config.unwrap()
                };
                interaction::offer_to_save_new_default_list(
                    // AddTaskValidated guarantees context.config is Some
                    &new_config,
                    &context.config_file_path,
                )?;
            }
        } else {
            self.add_task()?;
        }
        Ok(RunType::Completed("Task added".into()))
    }

    /// returns true if the default list should be offered to be saved
    fn add_task(self) -> AnyhowResult<bool> {
        self.client
            .add_task(self.list_id, &self.task)
            .map(|_| ())
            .map_err(|e| anyhow!(e))
            .context("Could not add task")?;

        Ok(self.possible_new_default_list)
    }

    fn user_message(&self) -> ColourOutput {
        ColourOutput::new(StreamKind::Stdout)
            .append("Adding task ", Style::Normal)
            .append(&self.task.content, Style::TaskContent)
            .append(format!(" to {}", &self.location_name), Style::Normal)
    }
}

#[derive(Debug)]
enum AddTaskError {
    UserCancellation,
    Unhandled(anyhow::Error),
}

impl From<&ConfigAbsentError> for AddTaskError {
    fn from(value: &ConfigAbsentError) -> Self {
        match value {
            ConfigAbsentError::UserCancellation => Self::UserCancellation,
            ConfigAbsentError::InteractionDisallowed => {
                Self::Unhandled(anyhow!("-q flag and no config file"))
            }
        }
    }
}

impl From<anyhow::Error> for AddTaskError {
    fn from(value: anyhow::Error) -> Self {
        Self::Unhandled(value)
    }
}

impl From<io::Error> for AddTaskError {
    fn from(value: io::Error) -> Self {
        Self::Unhandled(anyhow!(value))
    }
}

impl std::fmt::Display for AddTaskError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AddTaskError::UserCancellation => write!(f, "User cancelled add task operation"),
            AddTaskError::Unhandled(e) => write!(f, "{}", e),
        }
    }
}

fn is_content_piped() -> bool {
    atty::isnt(atty::Stream::Stdin)
}

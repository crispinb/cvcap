use std::io::{self, Read};

use anyhow::{anyhow, Context as ErrContext, Result};
use clap::Args;
use cvapi::{CheckvistClient, Task};
use dialoguer::{Confirm, Select};
use log::error;

use super::{Action, RunType};
use crate::app::{self, config, context, creds};
use crate::clipboard;
use crate::colour_output::{ColourOutput, StreamKind, Style};
use crate::progress_indicator::ProgressIndicator;

#[derive(Debug, Args)]
pub struct Add {
    #[clap(
        name = "task text",
        required_unless_present = "from clipboard",
        required_unless_present = "from stdin"
    )]
    task_content: Option<String>,
    /// Choose a list to add a new task to (ie. other than your default list)
    #[clap(
        name = "choose list",
        short = 'l',
        long,
        conflicts_with = "quiet",
        conflicts_with = "bookmark"
    )]
    choose_list: bool,
    /// Add a task from the clipboard instead of the command line
    #[clap(
        short = 'c',
        long,
        name = "from clipboard",
        conflicts_with = "task text",
        conflicts_with = "from stdin"
    )]
    from_clipboard: bool,
    /// Add a task from stdin instead of the command line
    #[clap(
        name = "from stdin",
        short = 's',
        long,
        conflicts_with = "task text",
        conflicts_with = "from clipboard"
    )]
    from_stdin: bool,
    /// Add task to custom location instead of top of default list
    /// (bookmark must exist in config file)
    #[clap(short = 'b', long = "bookmark", conflicts_with = "choose list")]
    pub bookmark: Option<String>,
}

impl Action for Add {
    fn run(self, context: context::Context) -> Result<RunType> {
        self.add_task(context)
    }
}

impl Add {
    // Create a new add action with a content string and all options false
    pub fn new(task_content: &str) -> Self {
        Self {
            task_content: Some(task_content.to_string()),
            choose_list: false,
            from_clipboard: false,
            from_stdin: false,
            bookmark: None,
        }
    }

    fn add_task(&self, context: context::Context) -> Result<RunType> {
        let api_token = match context.api_token {
            Some(token) => token,
            None => self.login_user(&context)?,
        };
        let client = CheckvistClient::new(
            &context.service_url,
            &api_token,
            #[allow(clippy::unit_arg)]
            Box::new(move |token| {
                creds::save_api_token_to_keyring(&context.keychain_service_name.clone(), token)
                    .unwrap_or(error!("Couldn't save token to keyring"))
            }),
        );

        let (appconfig, save_config) = match (context.config.clone(), self.choose_list) {
            // Prior config, no -l
            (Some(appconfig), false) => (appconfig, false),

            // prior config & -l
            (_, _) => {
                let bookmarks = if let Some(appconfig) = context.config {
                    appconfig.bookmarks
                } else {
                    None
                };
                match prompt_for_list(&client) {
                    Some((list_id, list_name, save_as_default)) => (
                        config::Config {
                            list_id,
                            list_name,
                            bookmarks,
                        },
                        save_as_default,
                    ),
                    None => return Ok(RunType::Cancelled),
                }
            }
        };

        if save_config {
            appconfig
                .write_to_new_file(&context.config_file_path)
                .with_context(|| {
                    format!(
                        "Couldn't save config file to path {:?}",
                        &context.config_file_path
                    )
                })?;
            ColourOutput::new(StreamKind::Stdout)
                .append(&appconfig.list_name, Style::ListName)
                .append(" is now your default list", Style::Normal)
                .println()?;
        }

        let content = match self.get_task_content(context.run_interactively)? {
            Some(content) => content,
            None => return Ok(RunType::Cancelled),
        };

        let (list_id, parent_id): (u32, Option<u32>) = match &self.bookmark {
            Some(bookmark_name) => match appconfig.bookmark(bookmark_name) {
                Ok(bookmark) => match bookmark {
                    Some(bookmark) => (bookmark.list_id, bookmark.parent_task_id),
                    None => Err(app::Error::BookmarkMissingError(bookmark_name.into()))?,
                },
                Err(e) => return Err(e),
            },
            None => (appconfig.list_id, None),
        };

        let task = Task {
            id: None,
            parent_id,
            content,
            position: 1,
        };

        let (dest_label, dest_name) = if let Some(bookmark) = &self.bookmark {
            (" to bookmark ".to_string(), bookmark.clone())
        } else {
            (" to list ".to_string(), appconfig.list_name)
        };

        let before_add_task = || {
            ColourOutput::new(StreamKind::Stdout)
                .append("Adding task ", Style::Normal)
                .append(&task.content, Style::TaskContent)
                .append(&dest_label, Style::Normal)
                .append(&dest_name, Style::ListName)
                .println()
                .expect("Problem printing colour output");
        };

        let add_task = || {
            client
                .add_task(list_id, &task)
                .map(|_| {
                    if context.run_interactively {
                        println!("\nTask added")
                    }
                })
                .map_err(|e| anyhow!(e))
                .context("Could not add task")
        };

        if context.run_interactively {
            ProgressIndicator::new('.', Box::new(before_add_task), 250).run(add_task)?;
        } else {
            add_task()?;
        }

        Ok(RunType::Completed)
    }

    // leave room here for future option to log in with username & password
    // as args
    fn login_user(&self, context: &context::Context) -> Result<String> {
        if context.run_interactively {
            creds::login_user(&context.keychain_service_name)
        } else {
            Err(anyhow!(app::Error::LoggedOut))
        }
    }

    /// Get content from args, clipboard or std, depending on user-provided options
    /// Ok(None) return indicates user cancellation
    fn get_task_content(&self, is_interactive: bool) -> Result<Option<String>> {
        match (self.from_clipboard, self.from_stdin) {
            (true, false) => self.get_content_from_clipboard(is_interactive),
            (false, true) => self.get_content_from_stdin(),
            (false, false) => Ok(self.task_content.clone()),
            (true, true) => panic!("Argument parsing failed"),
        }
    }

    fn get_content_from_stdin(&self) -> Result<Option<String>> {
        if !is_content_piped() {
            return Err(anyhow!(app::Error::MissingPipe));
        }
        let mut buffer = String::new();

        io::stdin().lock().read_to_string(&mut buffer)?;
        Ok(Some(buffer))
    }

    fn get_content_from_clipboard(&self, is_interactive: bool) -> Result<Option<String>> {
        let Some(cliptext) = clipboard::get_clipboard_as_string() else {
            return Err(anyhow!("Couldn't get clipboard contents"));
        };
        if !is_interactive
            || Confirm::new()
                .with_prompt(format!(r#"Add "{}" as a new task?"#, cliptext))
                .interact()?
        {
            Ok(Some(cliptext))
        } else {
            // indicates cancellation
            Ok(None)
        }
    }
}

/// returns (listid, list name, save list to new config)
fn prompt_for_list(client: &CheckvistClient) -> Option<(u32, String, bool)> {
    let lists_available = get_lists(client).ok()?;
    let Some((list_id, list_name)) = select_list(lists_available) else {
        return None;
    };
    let save_list_as_new_default = Confirm::new()
        .with_prompt(format!(
            "Do you want to save '{}' as your default list for future task capture?",
            list_name
        ))
        .interact()
        .ok()?;
    Some((list_id, list_name, save_list_as_new_default))
}

fn select_list(lists: Vec<(u32, String)>) -> Option<(u32, String)> {
    println!("Use arrow keys (or j/k) to pick a list. Enter/Space to choose. ESC to cancel\n");
    {
        let lists: &[(u32, String)] = &lists;
        let ids: Vec<&str> = lists.iter().map(|list| list.1.as_str()).collect();
        Select::new()
            .items(&ids)
            .interact_opt()
            // discard error here - nothing we can do so log & continue with None
            .map_err(|e| error!("{:?}", e))
            .ok()
            .flatten()
            // get list id and name as Ok val
            .map(|index| {
                lists
                    .get(index)
                    // if expect isn't safe here it's a lib (dialoguer) bug
                    .expect("Internal error getting list from user")
                    .to_owned()
            })
    }
    .map(|list| {
        ColourOutput::new(StreamKind::Stdout)
            .append("You picked list '", Style::Normal)
            .append(&list.1, Style::ListName)
            .println()
            .expect("Problem printing colour output");

        list
    })
}

fn is_content_piped() -> bool {
    atty::isnt(atty::Stream::Stdin)
}

fn get_lists(client: &CheckvistClient) -> Result<Vec<(u32, String)>> {
    let before_get_lists = || println!("Fetching lists from checkvist");

    let mut available_lists: Vec<(u32, String)> = Vec::new();
    ProgressIndicator::new('.', Box::new(before_get_lists), 250).run(|| {
        client
            .get_lists()
            .map(|lists| {
                available_lists = lists.into_iter().map(|list| (list.id, list.name)).collect()
            })
            .map_err(|e| anyhow!(e))
    })?;

    Ok(available_lists)
}

use crate::app::{
    self,
    cmd::{self, Action},
    creds,
};
use crate::colour_output::{ColourOutput, StreamKind, Style};
use crate::progress_indicator::ProgressIndicator;
use anyhow::{anyhow, Context, Error, Result};
use clap::Args;
use copypasta::{ClipboardContext, ClipboardProvider};
use cvcap::{CheckvistClient, Task};
use dialoguer::{theme::ColorfulTheme, Confirm, Select};
use log::error;
use std::io::{self, Read};

#[derive(Debug, Args)]
pub struct Add {
    #[clap(
        name = "task text",
        required_unless_present = "from clipboard",
        required_unless_present = "from stdin"
    )]
    task_content: Option<String>,
    /// Choose a list to add a new task to (ie. other than your default list)
    #[clap(short = 'l', long, conflicts_with = "quiet")]
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
}

impl Action for Add {
    fn run(self, context: app::Context) -> Result<cmd::RunType> {
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
        }
    }

    fn add_task(&self, context: app::Context) -> Result<cmd::RunType> {
        let api_token = match context.api_token {
            Some(token) => token,
            None => self.login_user(context.run_interactively)?,
        };
        let client = CheckvistClient::new(
            "https://checkvist.com/".into(),
            api_token,
            // clippy warns about the unit argument, but I want it for the side effect
            #[allow(clippy::unit_arg)]
            |token| {
                creds::save_api_token_to_keyring(token)
                    .unwrap_or(error!("Couldn't save token to keyring"))
            },
        );
        let config = match (context.config.clone(), self.choose_list) {
            (Some(config), false) => config,
            _ => match prompt_for_config(&client)? {
                Some(config) => config,
                None => return Ok(cmd::RunType::Cancelled),
            },
        };
        let content = match self.get_task_content(context.run_interactively)? {
            Some(content) => content,
            None => return Ok(cmd::RunType::Cancelled),
        };
        let task = Task {
            id: None,
            content,
            position: 1,
        };

        let before_add_task = || {
            let o = ColourOutput::new(StreamKind::Stdout)
                .append("Adding task ", Style::Normal)
                .append(&task.content, Style::TaskContent)
                .append(" to list ", Style::Normal)
                .append(&config.list_name, Style::ListName)
                .println();
        };

        let add_task = || {
            client
                .add_task(config.list_id, &task)
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

        Ok(cmd::RunType::Completed)
    }

    // leave room here for future option to log in with username & password
    // as args
    fn login_user(&self, is_interactive: bool) -> Result<String> {
        if is_interactive {
            creds::login_user()
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

    fn get_content_from_clipboard(&self, is_interactive: bool) -> Result<Option<String>> {
        let box_err_converter = |e| anyhow!("Error getting clipboard text: {:?}", e);
        let mut ctx = ClipboardContext::new().map_err(box_err_converter)?;
        let cliptext = ctx.get_contents().map_err(box_err_converter)?;
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

    fn get_content_from_stdin(&self) -> Result<Option<String>> {
        if !is_content_piped() {
            return Err(anyhow!(app::Error::MissingPipe));
        }
        let mut buffer = String::new();
        io::stdin().read_to_string(&mut buffer)?;
        Ok(Some(buffer))
    }
}

fn prompt_for_config(client: &CheckvistClient) -> Result<Option<app::Config>, Error> {
    let available_lists = get_lists(client)?;
    if let Some(user_config) = select_list(available_lists) {
        if Confirm::new()
            .with_prompt(format!(
                "Do you want to save '{}' as your default list for future task capture?",
                user_config.list_name
            ))
            .interact()?
        {
            user_config.write_to_new_file().with_context(|| {
                format!(
                    "Couldn't save config file to path {:?}",
                    app::config::config_file_path()
                )
            })?;
            let o = ColourOutput::new(StreamKind::Stdout);
            o.append(user_config.list_name.to_string(), Style::ListName)
                .append(" is now your default list", Style::Normal)
                .println();
        }

        Ok(Some(user_config))
    } else {
        Ok(None)
    }
}

fn select_list(lists: Vec<(u32, String)>) -> Option<app::Config> {
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
            .println();

        app::Config {
            list_id: list.0,
            list_name: list.1,
        }
    })
}

fn is_content_piped() -> bool {
    atty::isnt(atty::Stream::Stdin)
}

fn get_lists(client: &CheckvistClient) -> Result<Vec<(u32, String)>, Error> {
    let before_get_lists = || {
        println!("Fetching lists from checkvist")
    };

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

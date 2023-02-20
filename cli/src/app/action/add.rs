use std::io::{self, Read};

use anyhow::{anyhow, Context as ErrContext, Result};
use clap::Args;
use dialoguer::Confirm;

use super::{Action, RunType};
use crate::app::{self, context, interaction};
use crate::clipboard;
use crate::colour_output::{ColourOutput, StreamKind, Style};
use crate::progress_indicator::ProgressIndicator;
use cvapi::Task;

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
        let content = match self.get_task_content(context.allow_interaction)? {
            Some(content) => content,
            None => return Ok(RunType::Cancelled),
        };

        let Ok(config) = &context.config else {
            return Ok( RunType::Cancelled );
        };

        let (mut list_id, parent_id): (u32, Option<u32>) = match &self.bookmark {
            Some(bookmark_name) => match config.bookmark(bookmark_name) {
                Some(bookmark) => (bookmark.location.list_id, bookmark.location.parent_task_id),
                None => Err(app::Error::Reportable(format!("You tried to add a task to the bookmark '{}', but no bookmark of that name was found", bookmark_name)))?,
            },
            None => (config.list_id, None),
        };

        let task = Task {
            id: None,
            parent_id,
            content,
            position: 1,
        };

        let client = context.api_client()?;

        let mut use_non_default_list = false;
        // we now need a mutable config to potentially save to
        let mut config = context.config.unwrap();
        if self.choose_list {
            let Some(list) = interaction::user_select_list(&client, &format!( "Current default list: {}", config.list_name ))? else {
            return Ok(RunType::Cancelled);
        };
            // potentially save if this isn't the currently configured default list
            if list_id != list.0 {
                use_non_default_list = true;
            }
            list_id = list.0;
            config.list_id = list.0;
            config.list_name = list.1;
        }

        let (dest_label, dest_name) = if let Some(bookmark) = &self.bookmark {
            (" to bookmark ".to_string(), bookmark.clone())
        } else {
            (" to list ".to_string(), config.list_name.to_string())
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
                .map(|_| ())
                .map_err(|e| anyhow!(e))
                .context("Could not add task")
        };

        if context.allow_interaction {
            ProgressIndicator::new('.', Box::new(before_add_task), 250).run(add_task)?;
            if use_non_default_list {
                interaction::offer_to_save_new_default_list(&config, &context.config_file_path)?;
            }
        } else {
            add_task()?;
        }

        Ok(RunType::Completed("Task added".into()))
    }

    /// Get content from args, clipboard or std, depending on user-provided options
    /// Ok(None) return indicates user cancellation
    fn get_task_content(&self, allow_interaction: bool) -> Result<Option<String>> {
        match (self.from_clipboard, self.from_stdin) {
            (true, false) => self.get_content_from_clipboard(allow_interaction),
            (false, true) => self.get_content_from_stdin(),
            (false, false) => Ok(self.task_content.clone()),
            (true, true) => panic!("Argument parsing failed"),
        }
    }

    fn get_content_from_stdin(&self) -> Result<Option<String>> {
        if !is_content_piped() {
            return Err(anyhow!(app::Error::Reportable(
                "Tried to read from stdin pipe, but nothing was piped".into()
            )));
        }
        let mut buffer = String::new();

        io::stdin().lock().read_to_string(&mut buffer)?;
        Ok(Some(buffer))
    }

    fn get_content_from_clipboard(&self, allow_interaction: bool) -> Result<Option<String>> {
        let Some(cliptext) = clipboard::get_clipboard_as_string() else {
            return Err(anyhow!("Couldn't get clipboard contents"));
        };
        if allow_interaction
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

fn is_content_piped() -> bool {
    atty::isnt(atty::Stream::Stdin)
}

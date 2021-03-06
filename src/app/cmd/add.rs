use crate::app::{
    self,
    cmd::{self, Action},
    creds,
};
use crate::progress_indicator::ProgressIndicator;
use anyhow::{anyhow, Context, Error, Result};
use clap::Args;
use copypasta::{ClipboardContext, ClipboardProvider};
use cvcap::{CheckvistClient, Task};
use dialoguer::{Confirm, Select};
use log::error;

#[derive(Debug, Args)]
pub struct Add {
    #[clap(name = "task text", required_unless_present = "from-clipboard")]
    task_content: Option<String>,
    /// Choose a list to add a new task to (ie. other than your default list)
    #[clap(short = 'l', long)]
    choose_list: bool,
    /// Add a task from the clipboard instead of the command line
    #[clap(short = 'c', long, conflicts_with = "task text")]
    from_clipboard: bool,
}

impl Action for Add {
    fn run(self, context: app::Context) -> Result<cmd::RunType> {
        self.add_task(context)
    }
}

impl Add {
    fn add_task(&self, context: app::Context) -> Result<cmd::RunType> {
        let api_token = match context.api_token {
            Some(token) => token,
            None => creds::login_user()?,
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
            _ => prompt_for_config(&client)?,
        };

        let content = match self.content_from_args_or_clipboard()? {
            Some(content) => content,
            None => return Ok(cmd::RunType::Cancelled),
        };

        let task = Task {
            id: None,
            content,
            position: 1,
        };

        let add_task_msg = format!(
            r#"Adding task "{}" to list "{}""#,
            task.content, config.list_name
        );

        ProgressIndicator::new(".", &add_task_msg, "Task added", 250)
            .run(|| {
                client
                    .add_task(config.list_id, &task)
                    .map(|_t| ())
                    .map_err(|e| Box::new(e) as _)
            })
            // TODO - RESEARCH NEEDED: cf -------> I don't understand. The Anyhow
            // docs claim a From implementation for Box<dyn stdErr + Send + Sync +
            // 'static>, but I get a type mismatch  (which is what the map_err is a
            // workaround for)
            .map(|_| cmd::RunType::Continued)
            .map_err(|e| anyhow!(e))
            .context("Could not add task")
    }

    fn content_from_args_or_clipboard(&self) -> Result<Option<String>> {
        if !self.from_clipboard {
            return Ok(Some(self.task_content.as_ref().unwrap().clone()));
        };
        let box_err_converter = |e| anyhow!("Error getting clipboard text: {:?}", e);
        let mut ctx = ClipboardContext::new().map_err(box_err_converter)?;
        let cliptext = ctx.get_contents().map_err(box_err_converter)?;
        if Confirm::new()
            .with_prompt(format!(r#"Add "{}" as a new task?"#, cliptext))
            .interact()?
        {
            Ok(Some(cliptext))
        } else {
            Ok(None)
        }
    }
}

fn prompt_for_config(client: &CheckvistClient) -> Result<app::Config, Error> {
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
            println!("'{}' is now your default list", user_config.list_name);
        }

        Ok(user_config)
    } else {
        return Err(anyhow!("Could not collect config info from user"));
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
        println!("You picked list '{}'", list.1);
        app::Config {
            list_id: list.0,
            list_name: list.1,
        }
    })
}

fn get_lists(client: &CheckvistClient) -> Result<Vec<(u32, String)>, Error> {
    let mut available_lists: Vec<(u32, String)> = Vec::new();
    ProgressIndicator::new(".", "Fetching lists from Checkvist ", "", 250)
        .run(|| {
            available_lists = client
                .get_lists()
                .map(|lists| lists.into_iter().map(|list| (list.id, list.name)).collect())?;
            Ok(())
        })
        .map_err(|e| anyhow!(e))
        .context("Could not get lists from Checkvist API")?;
    Ok(available_lists)
}

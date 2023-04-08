//! Adds a bookmark, honouring -q by returning errors where interactions are required.
// NB: I'd like to find a better/more-principled means of handling interactions/-q
use anyhow::{anyhow, Context as AnyhowContext, Result as AnyhowResult};
use bpaf::{command, construct, params::ParseCommand, positional, Parser};
use cvapi::CheckvistError;
use dialoguer::Confirm;

use super::{
    context::{self, Context},
    Action, RunType,
};
use crate::{
    app::{bookmark::Bookmark, cli::Command, Error as AppError},
    config,
    progress_indicator::ProgressIndicator,
};

type Result<T> = std::result::Result<T, AddBookmarkError>;

#[derive(Debug, Clone)]
pub struct AddBookmark {
    pub name: String,
}

impl AddBookmark {
    pub fn command() -> ParseCommand<Command> {
        let name = positional("BOOKMARK_NAME")
            .help("The name to give to the bookmark\nThe bookmark's Checkvist URL must already be on the system clipboard");
        let add_bookmark_command = construct!(AddBookmark { name });
        let add_bookmark = construct!(Command::AddBookmark(add_bookmark_command))
            .to_options()
            .descr("Create bookmark, for adding tasks to lists other than the default list");
        command("add-bookmark", add_bookmark).help("Adds a Checkvist bookmark from the clipboard")
    }
}

impl Action for AddBookmark {
    fn run(self, context: Context) -> AnyhowResult<RunType> {
        match self.create_job(&context) {
            Ok(job) => job.run(context),
            Err(AddBookmarkError::UserCancellation) => Ok(RunType::Cancelled),
            Err(AddBookmarkError::Unhandled(err)) => Err(err),
        }
    }
}

// This isn't strictly an error type - eg. UserCancellation isn't
// an error, so is converted to Result<Runtype> before return in
// ::run. Using it here to keep nesting to a minimum (ie. using `?`).
// I suppose we can consider them errors in each function's context,
// even if they are not from the global module pov.
enum AddBookmarkError {
    UserCancellation,
    Unhandled(anyhow::Error),
}

impl From<context::ConfigAbsentError> for AddBookmarkError {
    fn from(value: context::ConfigAbsentError) -> Self {
        match value {
            context::ConfigAbsentError::UserCancellation => Self::UserCancellation,
            err => Self::Unhandled(anyhow!(err)),
        }
    }
}

impl From<anyhow::Error> for AddBookmarkError {
    fn from(value: anyhow::Error) -> Self {
        Self::Unhandled(value)
    }
}

impl From<CheckvistError> for AddBookmarkError {
    fn from(value: CheckvistError) -> Self {
        Self::Unhandled(value.into())
    }
}

impl From<std::io::Error> for AddBookmarkError {
    fn from(value: std::io::Error) -> Self {
        Self::Unhandled(value.into())
    }
}

impl AddBookmark {
    /// Get responses necessary to proceed with the add operation
    /// If context.allow_interactive is false, errors are returned if interactions
    /// would be needed for the add to proceed.
    fn create_job(self, context: &Context) -> Result<AddBookmarkJob> {
        let config = context.config.clone()?;
        let bookmark = Bookmark::from_clipboard(&self.name).with_context(|| {
            AppError::Reportable("The clipboard doesn't contain a valid bookmark".into())
        })?;
        // oh thank you I love you so much `Cargo fmt`
        let (bookmark_exists, replace_msg) = if config.find_bookmark_by_name(&self.name).is_some() {
            (
                true,
                format!(
                    "A Bookmark named {} already exists. Do you want to overwrite it?",
                    bookmark.name
                ),
            )
        } else if config
            .find_bookmark_by_location(&bookmark.location)
            .is_some()
        {
            (true, format!("A bookmark pointing to this Checkvist location already exists. Do you want to rename it to {}", bookmark.name))
        } else {
            (false, "".into())
        };

        if bookmark_exists {
            Self::ask_user_if_bookmark_should_be_replaced(context.allow_interaction, &replace_msg)?;
        }

        let client = context.api_client()?;

        let job = || {
            if !client.is_location_valid(&bookmark.location)? {
                return Err(anyhow!(AppError::Reportable(
                    "Checkvist reports that the requested bookmark location doesn't exist".into()
                )));
            };
            Ok(())
        };
        if context.allow_interaction {
            let p = ProgressIndicator::new(
                '.',
                Box::new(|| println!("\nChecking bookmark location is valid")),
                200,
            );
            p.run(job)?;
        } else {
            job()?;
        }

        Ok(AddBookmarkJob { config, bookmark })
    }

    fn ask_user_if_bookmark_should_be_replaced(
        allow_interaction: bool,
        replace_msg: &str,
    ) -> Result<()> {
        if !allow_interaction {
            return Err(AddBookmarkError::Unhandled(anyhow!(
                "the bookmark exists and interaction isn't allowed (probably `-q` flag set)"
            )));
        }
        if !Confirm::new().with_prompt(replace_msg).interact()? {
            return Err(AddBookmarkError::UserCancellation);
        };
        Ok(())
    }
}

// Validated data needed to add bookmark
pub struct AddBookmarkJob {
    bookmark: Bookmark,
    config: config::Config,
}

impl AddBookmarkJob {
    fn run(self, context: Context) -> AnyhowResult<RunType> {
        if context.allow_interaction {
            let p = ProgressIndicator::new('.', Box::new(|| println!("\nAdding bookmark")), 100);
            p.run(|| self.add_bookmark(context))
        } else {
            self.add_bookmark(context)
        }
    }
    /// Add the bookmark in self.bookmark to the config in self.config
    /// This method does no user interaction
    fn add_bookmark(mut self, context: Context) -> AnyhowResult<RunType> {
        self.config.add_bookmark(self.bookmark, true)?;
        self.config.save(&context.config_file_path)?;

        Ok(RunType::Completed("\nBookmark Added".into()))
    }
}

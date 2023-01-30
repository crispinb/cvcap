//! Adds a bookmark, honouring -q by returning errors where interactions are required.
// NB: I'd like to find a better/more-principled means of handling interactions/-q
use anyhow::{anyhow, Result as AnyhowResult};
use clap::Args;
use cvapi::CheckvistError;
use dialoguer::Confirm;

use super::{
    context::{self, Context},
    Action, RunType,
};
use crate::{app::bookmark::Bookmark, progress_indicator::ProgressIndicator};

#[derive(Debug, Args)]
pub struct AddBookmark {
    /// The name to give to the bookmark
    /// The bookmark's Checkvist URL must already be on the system clipboard
    #[clap(name = "bookmark name")]
    name: String,
    #[clap(skip)] // clap ignore otherwise we need to impl Bookmark::FromStr
    bookmark: Option<Bookmark>,
}

type Result<T> = std::result::Result<T, AddBookmarkError>;

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

impl Action for AddBookmark {
    fn run(self, context: Context) -> AnyhowResult<RunType> {
        return match self.gather_user_responses(&context) {
            Ok(response) => {
                let allow_interaction = context.allow_interaction;
                let job = || response.add_bookmark(context);
                if allow_interaction {
                let p =
                    ProgressIndicator::new('.', Box::new(|| println!("\nAdding bookmark")), 200);
                p.run(job)
                } else {
                    job()
                }
            }
            Err(AddBookmarkError::UserCancellation) => Ok(RunType::Cancelled),
            Err(AddBookmarkError::Unhandled(err)) => Err(err),
        };
    }
}

impl AddBookmark {
    /// Get responses necessary to proceed with the add operation
    /// If context.allow_interactive is false, errors are returned if interactions
    /// would be needed for the add to proceed.
    fn gather_user_responses(mut self, context: &Context) -> Result<AddBookmark> {
        let config = context.config.clone()?;
        let bookmark = Bookmark::from_clipboard(&self.name)?;
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
                return Err(anyhow!(
                    "Checkvist reports that the requested bookmark location doesn't exist"
                ));
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

        self.bookmark = Some(bookmark);
        Ok(self)
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

    /// Add the bookmark in self.bookmark to the config in self.config
    /// self.bookmark: None is an error
    /// self.config: None just indicates that the user cancelled
    /// the config setup (see AddBookmarkError From impl)
    /// This method does no user interaction
    fn add_bookmark(self, context: Context) -> AnyhowResult<RunType> {
        let Ok(mut config) = context.config else {
             return Ok(RunType::Cancelled);
        };
        let Some(bookmark) = self.bookmark else {
            panic!("AddBookmark struct should have a bookmark");
        };

        config.add_bookmark(bookmark, true)?;
        config.save(&context.config_file_path)?;

        Ok(RunType::Completed("\nBookmark Added".into()))
    }
}

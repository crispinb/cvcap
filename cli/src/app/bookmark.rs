use std::fmt::Debug;

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use url::Url;

use super::Error;
use crate::clipboard;
use cvapi::CheckvistLocation;

#[derive(Clone, Deserialize, Serialize, PartialEq, Debug)]
pub struct Bookmark {
    pub name: String,
    pub location: CheckvistLocation,
}

impl Bookmark {
    pub fn from_clipboard(name: &str) -> Result<Bookmark> {
        let cliptext = clipboard::get_clipboard_as_string()
            .ok_or(anyhow!("Couldn't get text from the clipboard"))?;
        let bookmark = Bookmark::try_from(cliptext.as_str())?;
        Ok(bookmark.rename(name))
    }

    pub fn rename(mut self, name: &str) -> Self {
        self.name = name.to_string();
        self
    }
}

impl TryFrom<&str> for Bookmark {
    type Error = Error;

    /// Attempt to parse a bookmark from a (presumably Checkvist) url.
    /// This encodes the list id and parent task id (if any), but not
    /// the name, so the latter is set as UNNAMED
    fn try_from(s: &str) -> Result<Self, Error> {
        let bookmark_url = Url::parse(s).map_err(|_| Error::InvalidBookmarkStringFormat)?;
        let url_segments = bookmark_url
            .path_segments()
            .map(|s| s.collect::<Vec<_>>())
            .ok_or(Error::InvalidBookmarkStringFormat)?;
        let (list_id_str, parent_task_id) = match url_segments[..] {
            ["checklists", list_id_str] => (list_id_str, None),
            ["checklists", list_id_str, "tasks", task_idstr] => {
                let parent_task_id: u32 = task_idstr
                    .parse()
                    .map_err(|_| Error::InvalidBookmarkStringFormat)?;
                (list_id_str, Some(parent_task_id))
            }
            _ => return Err(Error::InvalidBookmarkStringFormat),
        };
        let list_id: u32 = list_id_str
            .parse()
            .map_err(|_| Error::InvalidBookmarkStringFormat)?;

        let location = CheckvistLocation {
            list_id,
            parent_task_id,
        };
        Ok(Bookmark {
            name: "UNNAMED".to_string(),
            location,
        })
    }
}

#[cfg(test)]
mod test {
    // use serde::{Deserialize, Serialize};
    use super::*;
    use copypasta::{ClipboardContext, ClipboardProvider};
    use serial_test::serial;

    /// ClipboardContext needs exclusive access, so we must serialise these tests
    #[test]
    #[serial]
    fn get_valid_bookmark_from_clipboard() {
        let cliptext = "https://checkvist.com/checklists/1/tasks/2".to_string();
        let mut clip_ctx = ClipboardContext::new().unwrap();
        clip_ctx.set_contents(cliptext).unwrap();

        let bookmark = Bookmark::from_clipboard("bm1").unwrap();

        assert_eq!(bookmark.location.list_id, 1);
        assert_eq!(bookmark.location.parent_task_id, Some(2));
        assert_eq!(bookmark.name, "bm1");
    }

    #[test]
    #[serial]
    fn get_from_invalid_clipboard_contents_errors() {
        let cliptext = "".to_string();
        let mut clip_ctx = ClipboardContext::new().unwrap();
        clip_ctx.set_contents(cliptext).unwrap();

        let error = Bookmark::from_clipboard("bm1").expect_err("Expected an err");
        let error_cause = error
            .root_cause()
            .downcast_ref::<crate::app::Error>()
            .unwrap();

        assert!(error.is::<crate::app::Error>());
        assert!(std::matches!(
            error_cause,
            crate::app::Error::InvalidBookmarkStringFormat
        ));
    }

    // #[test]
    // fn bookmark_equality() {
    //     let bm = Bookmark {
    //         name: "name".into(),
    //         list_id: 1,
    //         parent_task_id: Some(1),
    //     };
    //     let copy = bm.clone();
    //     let mut renamed = bm.clone();
    //     renamed.name = "noname".into();
    //     let mut relisted = bm.clone();
    //     relisted.list_id = 2;
    //     let mut retasked = bm.clone();
    //     retasked.parent_task_id = Some(2);
    //     let mut untasked = bm.clone();
    //     untasked.parent_task_id = None;

    //     assert_eq!(bm.difference(&copy), BookmarkComparison::Same);
    //     assert_eq!(bm.difference(&renamed), BookmarkComparison::Name);
    //     assert_eq!(bm.difference(&relisted), BookmarkComparison::List);
    //     assert_eq!(bm.difference(&retasked), BookmarkComparison::Task);
    //     assert_eq!(bm.difference(&untasked), BookmarkComparison::Task);
    // }
}

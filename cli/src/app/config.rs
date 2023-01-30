use std::fs;
use std::path::PathBuf;

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

use super::{bookmark::Bookmark, Error};
use cvapi::CheckvistLocation;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Config {
    #[serde(rename = "default_list_id")]
    pub list_id: u32,
    #[serde(rename = "default_list_name")]
    pub list_name: String,
    pub bookmarks: Option<Vec<Bookmark>>,
}

impl Config {
    pub fn from_file(path: &PathBuf) -> Result<Option<Self>> {
        if !path.is_file() {
            return Ok(None);
        }
        let config_file = fs::read_to_string(path)?;
        let config = Some(
            toml::from_str(&config_file)
                .map_err(|_e| Error::InvalidConfigFile(path.to_string_lossy().into()))?,
        );
        Ok(config)
    }

    pub fn save(&self, path: &PathBuf) -> Result<()> {
        let config_dir = path.parent().expect("Couldn't construct config path");
        if !config_dir.is_dir() {
            fs::create_dir_all(config_dir)?;
        }

        let toml = toml::to_string(self)?;
        std::fs::write(path, toml)?;
        Ok(())
    }

    pub fn bookmark(&self, name: &str) -> Option<Bookmark> {
        let Some(bookmarks) = &self.bookmarks else {
            return None;
        };

        bookmarks.iter().find(|b| b.name == name).cloned()
    }

    /// Adds bookmark to config
    /// If bookmark with same name and/or location exists, it will
    /// be replaced if `replace` is true, otherwise an error
    /// is returned
    pub fn add_bookmark(&mut self, bookmark: Bookmark, replace: bool) -> Result<()> {
        // name equality takes precedence over location
        let existing = if let name_match @ Some(_index) = self.find_bookmark_by_name(&bookmark.name)
        {
            name_match
        } else {
            self.find_bookmark_by_location(&bookmark.location)
        };

        if self.bookmarks.is_none() {
            self.bookmarks = Some(Vec::new());
        };

        let bookmarks = self.bookmarks.as_mut().unwrap();

        match (existing, replace) {
            (Some(_existing), false) => Err(anyhow!(
                "Tried to replace an existing bookmark, with 'replace' false"
            )),
            (Some(existing), true) => {
                bookmarks[existing] = bookmark;
                Ok(())
            }

            _ => {
                bookmarks.push(bookmark);
                Ok(())
            }
        }
    }

    /// return the index of the bookmark with this name, or None
    pub fn find_bookmark_by_name(&self, name: &str) -> Option<usize> {
        match self.bookmarks {
            None => None,
            Some(ref bookmarks) => bookmarks
                .iter()
                .enumerate()
                .find(|(_index, bookmark)| bookmark.name == name)
                .map(|(index, _bm)| index),
        }
    }

    /// return the index of the bookmark with this location, or None
    pub fn find_bookmark_by_location(&self, location: &CheckvistLocation) -> Option<usize> {
        match self.bookmarks {
            None => None,
            Some(ref bookmarks) => bookmarks
                .iter()
                .enumerate()
                .find(|(_index, bookmark)| &bookmark.location == location)
                .map(|(index, _bm)| index),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use temp_dir::TempDir;

    #[test]
    fn read_config_file_without_bookmarks() {
        let config = Config {
            list_id: 1,
            list_name: "test_list".into(),
            bookmarks: None,
        };
        let t = TempDir::new().unwrap();
        let path = t.child("temp.toml");
        config.save(&path).unwrap();

        let read_config = Config::from_file(&path).unwrap().unwrap();

        assert_eq!(read_config, config);
    }

    #[test]
    fn read_config_file_with_bookmarks() {
        let t = TempDir::new().unwrap();
        let path = t.child("temp.toml");
        let list_location = CheckvistLocation {
            list_id: 1,
            parent_task_id: None,
        };
        let task_location = CheckvistLocation {
            list_id: 1,
            parent_task_id: Some(1),
        };
        let source_config = Config {
            list_id: 1,
            list_name: "test_list".into(),
            bookmarks: Some(vec![
                Bookmark {
                    name: "bm1".into(),
                    location: list_location,
                },
                Bookmark {
                    name: "bm2".into(),
                    location: task_location,
                },
            ]),
        };
        source_config.save(&path).unwrap();

        let config = Config::from_file(&path).unwrap().unwrap();
        let no_bookmark = config.bookmark("none");
        let list_bookmark = config.bookmark("bm1").unwrap();
        let task_bookmark = config.bookmark("bm2").unwrap();

        assert_eq!(config, source_config);
        assert!(no_bookmark.is_none());
        assert_eq!(list_bookmark.location.list_id, 1);
        assert_eq!(task_bookmark.location.parent_task_id.unwrap(), 1);
    }

    #[test]
    fn nonexistent_config_file_returns_error() {
        let t = TempDir::new().unwrap();
        let path = t.child("temp.toml");
        std::fs::write(&path, "This is no config file").unwrap();

        let result = Config::from_file(&path);
        assert!(result.is_err());
    }

    #[test]
    fn add_bookmarks() {
        let bookmark = Bookmark {
            name: "bm1".into(),
            location: CheckvistLocation {
                list_id: 1,
                parent_task_id: None,
            },
        };
        let mut config = Config {
            list_id: 1,
            list_name: "list".into(),
            bookmarks: None,
        };

        let result = config.add_bookmark(bookmark, false).unwrap();
        {
            let bookmarks = config.bookmarks.as_ref().unwrap();

            assert_eq!(result, ());
            assert_eq!(bookmarks.len(), 1usize);
            assert_eq!(bookmarks[0].location.list_id, 1);
        }

        let bookmark2 = Bookmark {
            name: "bm2".into(),
            location: CheckvistLocation {
                list_id: 2,
                parent_task_id: None,
            },
        };

        config.add_bookmark(bookmark2, false).unwrap();

        {
            let bookmarks = config.bookmarks.as_ref().unwrap();

            assert_eq!(bookmarks.len(), 2usize);
            assert_eq!(bookmarks[1].location.list_id, 2);
        }
    }

    #[test]
    fn add_existing_bookmark_errors_if_replace_not_specified() {
        let bookmark = Bookmark {
            name: "bm1".into(),
            location: CheckvistLocation {
                list_id: 1,
                parent_task_id: None,
            },
        };
        let bookmarks = vec![bookmark.clone()];
        let mut new_bookmark = bookmark;
        new_bookmark.name = "bm2".into();
        let mut config = Config {
            list_id: 1,
            list_name: "list".into(),
            bookmarks: Some(bookmarks),
        };

        let result = config.add_bookmark(new_bookmark, false);

        assert!(result.is_err());
    }

    #[test]
    fn add_can_replace_bookmark_with_same_name() {
        let existing = Bookmark {
            name: "bm1".into(),
            location: CheckvistLocation {
                list_id: 1,
                parent_task_id: None,
            },
        };
        let mut new = existing.clone();
        let new_location = CheckvistLocation {
            list_id: 2,
            parent_task_id: None,
        };
        new.location = new_location.clone();

        let bookmarks = vec![existing.clone()];
        let mut config = Config {
            list_id: 1,
            list_name: "list".into(),
            bookmarks: Some(bookmarks),
        };

        let _result = config.add_bookmark(new, true).unwrap();

        assert!(config.find_bookmark_by_location(&new_location).is_some());
    }
}

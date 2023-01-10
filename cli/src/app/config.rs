use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use url::Url;

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

use super::Error;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Config {
    #[serde(rename = "default_list_id")]
    pub list_id: u32,
    #[serde(rename = "default_list_name")]
    pub list_name: String,
    // TODO: so how to make this option in TOML? And why has it changed??
    pub bookmarks: Option<HashMap<String, String>>,
}

pub struct Bookmark {
    pub list_id: u32,
    pub parent_task_id: Option<u32>,
}

impl Config {
    pub fn from_file(path: &PathBuf) -> Result<Option<Self>> {
        // it's OK not to have a config file yet
        if !path.is_file() {
            return Ok(None);
        }
        let config_file = fs::read_to_string(path)?;
        let config = Some( toml::from_str(&config_file).map_err(|_e| Error::InvalidConfigFile(path.to_string_lossy().into()))?);
        Ok(config)
    }

    pub fn write_to_new_file(&self, path: &PathBuf) -> Result<()> {
        let config_dir = path.parent().expect("Couldn't construct config path");
        if !config_dir.is_dir() {
            fs::create_dir_all(config_dir)?;
        }

        let toml = toml::to_string(self)?;
        std::fs::write(path, toml)?;
        Ok(())
    }
    
    // TODO: replace with Config struct custom deserialiser/serialiser pair (otherwise gets clobbered on writing)
    pub fn bookmark(&self, name: &str) -> Result<Option<Bookmark>> {
        if let Some(bookmarks) = &self.bookmarks {
            if let Some(bookmark) = bookmarks.get(name) {
                let bookmark_url = Url::parse(bookmark).map_err(|_| Error::BookmarkFormatError)?;
                let url_segments = bookmark_url
                    .path_segments()
                    .map(|s| s.collect::<Vec<_>>())
                    .ok_or(Error::BookmarkFormatError)?;
                println!("segments: {:?}", url_segments);
                match url_segments[..] {
                    ["checklists", list_idstr] => {
                        let list_id: u32 =
                            list_idstr.parse().map_err(|_| Error::BookmarkFormatError)?;
                        Ok(Some(Bookmark {
                            list_id,
                            parent_task_id: None,
                        }))
                    }
                    ["checklists", list_idstr, "tasks", task_idstr] => {
                        let list_id: u32 =
                            list_idstr.parse().map_err(|_| Error::BookmarkFormatError)?;
                        let parent_task_id: u32 =
                            task_idstr.parse().map_err(|_| Error::BookmarkFormatError)?;
                        Ok(Some(Bookmark {
                            list_id,
                            parent_task_id: Some(parent_task_id),
                        }))
                    }
                    _ => Err(anyhow!(Error::BookmarkFormatError)),
                }
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
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
        config.write_to_new_file(&path).unwrap();

        let read_config = Config::from_file(&path).unwrap().unwrap();

        assert_eq!(read_config, config);
    }

    #[test]
    fn read_config_file_with_bookmarks() {
        let t = TempDir::new().unwrap();
        let path = t.child("temp.toml");
        let config = Config {
            list_id: 1,
            list_name: "test_list".into(),
            bookmarks: Some(HashMap::from([
                (
                    "list_bookmark".into(),
                    "https://beta.checkvist.com/checklists/1".into(),
                ),
                (
                    "task_bookmark".into(),
                    "https://beta.checkvist.com/checklists/1/tasks/1".into(),
                ),
            ])),
        };
        config.write_to_new_file(&path).unwrap();

        let read_config = Config::from_file(&path).unwrap().unwrap();
        let no_bookmark = read_config.bookmark("none").unwrap();
        let list_bookmark = read_config.bookmark("list_bookmark").unwrap().unwrap();
        let task_bookmark = read_config.bookmark("task_bookmark").unwrap().unwrap();

        assert_eq!(read_config, config);
        assert!(no_bookmark.is_none());
        assert_eq!(list_bookmark.list_id, 1);
        assert_eq!(task_bookmark.parent_task_id.unwrap(), 1);
    }

    #[test]
    fn bad_config_file_returns_error() {
        let t = TempDir::new().unwrap();
        let path = t.child("temp.toml");
        std::fs::write(&path, "This is no config file").unwrap();

        let result = Config::from_file(&path);
        assert!(result.is_err());
    }
}

use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use anyhow::Result;
use serde::{Deserialize, Serialize};

use super::{bookmark::Bookmark, Error};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Config {
    #[serde(rename = "default_list_id")]
    pub list_id: u32,
    #[serde(rename = "default_list_name")]
    pub list_name: String,
    pub bookmarks: Option<HashMap<String, String>>,
}

impl Config {
    pub fn from_file(path: &PathBuf) -> Result<Option<Self>> {
        // it's OK not to have a config file yet
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

    pub fn write_to_new_file(&self, path: &PathBuf) -> Result<()> {
        let config_dir = path.parent().expect("Couldn't construct config path");
        if !config_dir.is_dir() {
            fs::create_dir_all(config_dir)?;
        }

        let toml = toml::to_string(self)?;
        std::fs::write(path, toml)?;
        Ok(())
    }

    pub fn bookmark(&self, name: &str) -> Result<Option<Bookmark>> {
        let Some(bookmarks) = &self.bookmarks else {
            return Ok(None);
        };
        let Some(bookmark_string) = bookmarks.get(name) else {
            return Ok(None);
        };
        let bookmark: Bookmark = Bookmark::try_from(bookmark_string.as_str())?;
        Ok(Some(bookmark))
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

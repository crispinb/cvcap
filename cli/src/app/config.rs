use std::{env, fs, path};

use anyhow::Result;
use directories::ProjectDirs;
use log::error;
use serde::{Deserialize, Serialize};

const NON_DEFAULT_PATH_ENV_KEY: &str = "CVCAP_CONFIG_FILE_PATH";
const CONFIG_FILE_NAME: &str = "cvcap.toml";

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Config {
    #[serde(rename = "default_list_id")]
    pub list_id: u32,
    #[serde(rename = "default_list_name")]
    pub list_name: String,
    pub bookmarks: Option<Vec<(String, String)>>,
}

pub enum FilePathSource {
    Standard,
    Custom(path::PathBuf),
}

impl Config {
    pub fn read_from_file(source: &FilePathSource) -> Option<Self> {
        let config_file_path = match source {
            FilePathSource::Standard => config_file_path(source),
            FilePathSource::Custom(path) => path.to_path_buf(),
        };
        let config_file = fs::read_to_string(&config_file_path).ok()?;
        let config = toml::from_str(&config_file).ok().or_else(|| {
            error!(
                "Failed to parse config file at path {:?}. Continuing without",
                config_file_path
            );
            None
        })?;
        Some(config)
    }

    pub fn write_to_new_file(&self, source: &FilePathSource) -> Result<()> {
        let config_path = config_file_path(source);
        let config_dir = config_path
            .parent()
            .expect("Couldn't construct config path");
        if !config_dir.is_dir() {
            fs::create_dir_all(config_dir)?;
        }

        let toml = toml::to_string(self)?;
        std::fs::write(config_path, toml)?;
        Ok(())
    }
}

pub fn config_file_path(source: &FilePathSource) -> path::PathBuf {
    match source {
        FilePathSource::Standard => match env::var_os(NON_DEFAULT_PATH_ENV_KEY) {
            Some(path) => path::PathBuf::from(path),
            None => ProjectDirs::from("com", "not10x", "cvcap")
                .expect("OS cannot find HOME dir. Cannot proceed")
                .config_dir()
                .join(CONFIG_FILE_NAME),
        },
        FilePathSource::Custom(path) => path.to_path_buf(),
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
        let config_file_path = t.child("temp.toml");
        let source = FilePathSource::Custom(config_file_path);
        config.write_to_new_file(&source).unwrap();

        let read_config = Config::read_from_file(&source).unwrap();

        assert_eq!(read_config, config);
    }

    #[test]
    fn read_config_file_with_bookmarks() {
        let t = TempDir::new().unwrap();
        let config_file_path = t.child("temp.toml");
        let source = FilePathSource::Custom(config_file_path);
        let config = Config {
            list_id: 1,
            list_name: "test_list".into(),
            bookmarks: Some(vec![("bookmark1".into(), "testbm".into()),("bookmark2".into(), "testbm".into())]),
        };
        config.write_to_new_file(&source).unwrap();

        let read_config = Config::read_from_file(&source).unwrap();
        println!("----------> {:?}", &read_config);

        assert_eq!(read_config, config);
    }

    // TODO: coerce bookmark to url? or otherwise method on config to extract bookmarks' list_id
    // and task_id
}

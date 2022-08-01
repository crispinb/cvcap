use anyhow::Result;
use directories::ProjectDirs;
use log::error;
use serde::{Deserialize, Serialize};
use std::{env, fs, path};

const NON_DEFAULT_PATH_ENV_KEY: &str = "CVCAP_CONFIG_FILE_PATH";
const CONFIG_FILE_NAME: &str = "cvcap.toml";

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Config {
    #[serde(rename = "default_list_id")]
    pub list_id: u32,
    #[serde(rename = "default_list_name")]
    pub list_name: String,
}

impl Config {
    pub fn read_from_file() -> Option<Self> {
        let config_file = fs::read_to_string(config_file_path()).ok()?;
        let config = toml::from_str(&config_file).ok().or_else(|| {
            error!(
                "Failed to parse config file at path {:?}. Continuing without",
                config_file_path()
            );
            None
        })?;
        Some(config)
    }

    pub fn write_to_new_file(&self) -> Result<()> {
        let config_path = config_file_path();
        let config_dir = config_path
            .parent()
            .expect("Couldn't construct config path");
        if !config_dir.is_dir() {
            fs::create_dir_all(config_dir)?;
        }

        let toml = toml::to_string(self)?;
        std::fs::write(config_file_path(), toml)?;
        Ok(())
    }
}

pub fn config_file_path() -> path::PathBuf {
    match env::var_os(NON_DEFAULT_PATH_ENV_KEY) {
        Some(path) => path::PathBuf::from(path),
        None => ProjectDirs::from("com", "not10x", "cvcap")
            .expect("OS cannot find HOME dir. Cannot proceed")
            .config_dir()
            .join(CONFIG_FILE_NAME),
    }
}

// dupes of functions in bin::app::config
// May pull those out into a lib instead

use anyhow::Result;

use serde::{Deserialize, Serialize};
use std::fs;
use std::path;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TestCvcapRunConfig {
    #[serde(rename = "default_list_id")]
    pub list_id: u32,
    #[serde(rename = "default_list_name")]
    pub list_name: String,
}

impl TestCvcapRunConfig {
    pub fn write_to_new_file(&self, config_path: &path::PathBuf) -> Result<()> {
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

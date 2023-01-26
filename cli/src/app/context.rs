use std::env;
use std::path::PathBuf;

use anyhow::{anyhow, Result};
use directories::ProjectDirs;

use super::config::Config;
use super::creds;

/// TODO: document
// TODO: generalise standard-vs-via-env
const KEYCHAIN_SERVICE_NAME: &str = "cvcap-api-token";
pub const CUSTOM_SERVICE_NAME_ENV_KEY: &str = "CVCAP_CREDENTIAL_ID";
pub const CUSTOM_SERVICE_URL_KEY: &str = "CVCAP_SERVICE_URL";
pub const CUSTOM_CONFIG_FILE_PATH_ENV_KEY: &str = "CVCAP_CONFIG_FILE_PATH";
const CONFIG_FILE_NAME: &str = "cvcap.toml";

#[derive(Clone)]
pub struct Context {
    pub config: Option<Config>,
    pub config_file_path: PathBuf,
    pub api_token: Option<String>,
    pub service_url: String,
    // this becase I haven't found a way to get access to higher level Command args from a
    // subcommand. see https://github.com/crispinb/cvcap/issues/26
    pub run_interactively: bool,
    pub keychain_service_name: String,
}

impl Context {
    pub fn new(run_interactively: bool) -> Result<Self> {
        let config_file_path = Self::config_file_path();
        let config = match Config::from_file(&config_file_path) {
            Ok(config) => config,
            Err(e) => return Err(anyhow!(e)),
        };
        let service_url = match env::var_os(CUSTOM_SERVICE_URL_KEY) {
            Some(url) => url.to_string_lossy().into(),
            None => "https://checkvist.com".into(),
        };

        let keychain_service_name = Self::keychain_service_name();
        let api_token = creds::get_api_token_from_keyring(&keychain_service_name);
        Ok(Context {
            config,
            config_file_path,
            api_token,
            service_url,
            run_interactively,
            keychain_service_name,
        })
    }

    fn config_file_path() -> PathBuf {
        match env::var_os(CUSTOM_CONFIG_FILE_PATH_ENV_KEY) {
            // "expect" fine here as anyone using this will be able to cope
            Some(path) => path
                .into_string()
                .expect("Invalid custom config file path. Cannot proceed")
                .into(),
            None => ProjectDirs::from("com", "not10x", "cvcap")
                .expect("OS cannot find HOME dir. Cannot proceed")
                .config_dir()
                .join(CONFIG_FILE_NAME),
        }
    }

    fn keychain_service_name() -> String {
        match env::var_os(CUSTOM_SERVICE_NAME_ENV_KEY) {
            Some(name) => name.into_string().expect(
                "Couldn't get a valid credential key from CVCAP_CREDENTIAL_ID environment variable",
            ),
            None => KEYCHAIN_SERVICE_NAME.into(),
        }
    }
}

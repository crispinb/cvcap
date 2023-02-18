use std::path::PathBuf;
/// Provides configuration information needed for the cvcap app at runtime The base checkvist url,
/// path to the toml config file, and name for the OS-dependent keychain used for storing Checkvist
/// API credentials, can all be customised via env vars.
use std::{env, fmt::Display};

use anyhow::{anyhow, Result};
use cvapi::CheckvistClient;
use dialoguer::{Confirm, Input, Password};
use directories::ProjectDirs;
use log::error;

use super::{config::Config, creds, interaction};
use crate::colour_output::{ColourOutput, StreamKind, Style};

const KEYCHAIN_SERVICE_NAME: &str = "cvcap-api-token";
/// Environment variable to customise the name of the keychain or other OS-dependent
/// service used to store user credentials
pub const CUSTOM_SERVICE_NAME_ENV_KEY: &str = "CVCAP_CREDENTIAL_ID";
/// Environment variable for customising the base Checkvist API url
pub const CUSTOM_SERVICE_URL_KEY: &str = "CVCAP_SERVICE_URL";
/// Environment variable for customising the path to the cvcap config file (toml format)
pub const CUSTOM_CONFIG_FILE_PATH_ENV_KEY: &str = "CVCAP_CONFIG_FILE_PATH";
const CONFIG_FILE_NAME: &str = "cvcap.toml";

#[derive(Clone, Debug)]
#[non_exhaustive]
pub struct Context {
    /// config Result::Err indicates any reason for not having a valid config
    pub config: Result<Config, ConfigAbsentError>,
    pub config_file_path: PathBuf,
    // this becase I haven't found a way to get access to higher level Command args from a
    // subcommand. see https://github.com/crispinb/cvcap/issues/26
    pub allow_interaction: bool,
    // these are only needed for building a CheckvistClient,
    // but as that's tricky to make Cloneable (because of
    // the callback), we hold them rather than a prebuilt client
    pub api_token: Option<String>,
    pub checkvist_base_url: String,
    pub keychain_service_name: String,
}

#[derive(Debug, Clone)]
pub enum ConfigAbsentError {
    UserCancellation,
    InteractionDisallowed,
}

impl Display for ConfigAbsentError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigAbsentError::UserCancellation => write!(
                f,
                "no config file, and user cancelled request to set one up"
            ),
            ConfigAbsentError::InteractionDisallowed => write!(
                f,
                "no config file, and user interaction disallowed (probably `-q` was set)"
            ),
        }
    }
}

impl Context {
    /// Obtains and holds runtime resources: a valid config, commandline params that
    /// aren't associated with specific actions, and data needed to create api clients
    ///
    /// If a valid config file exists, `allow_interaction` has no effect. `context.config` will
    /// be populated via the existing file.
    ///
    /// If there is no current valid config file:
    ///     If `allow_interaction == true` the user will be prompted for Checkvist login
    ///     details if not currently logged in, and a new config file will be created.
    ///     If `allow_interaction == false` self.api_token will be None (ie. the user isn't logged in)
    ///                                     self.config will also be None
    pub fn new(allow_interaction: bool) -> Result<Self> {
        let service_url: String = match env::var_os(CUSTOM_SERVICE_URL_KEY) {
            Some(url) => url.to_string_lossy().into(),
            None => "https://checkvist.com".into(),
        };
        let keychain_service_name = Self::keychain_service_name();
        let api_token = match creds::get_api_token_from_keyring(&keychain_service_name) {
            Some(token) => Some(token),
            None if allow_interaction => {
                Some(Self::login_user(&service_url, &keychain_service_name)?)
            }
            _ => None,
        };

        let config_file_path = Self::config_file_path();

        let config = match (Config::from_file(&config_file_path)?, allow_interaction) {
            (Some(config), _) => Ok(config),
            (None, true) => {
                let checkvist_client = Self::api_client_interactive(
                    &service_url,
                    &keychain_service_name,
                    api_token.clone(),
                )?;
                Self::prompt_user_to_set_up_new_config(&checkvist_client, &config_file_path)?
                    .ok_or(ConfigAbsentError::UserCancellation)
            }
            (None, false) => Err(ConfigAbsentError::InteractionDisallowed),
        };

        Ok(Context {
            config,
            config_file_path,
            api_token,
            checkvist_base_url: service_url,
            allow_interaction,
            keychain_service_name,
        })
    }

    /// Builds and returns CheckvistClient, or error if user is not logged in
    pub fn api_client(&self) -> Result<CheckvistClient> {
        let Some(ref api_token) = self.api_token else {
            return Err(anyhow!("Cannot create CheckvistClient while logged out"));
        };
        let keychain_service_name = self.keychain_service_name.to_string();
        Ok(CheckvistClient::new(
            &self.checkvist_base_url,
            api_token,
            #[allow(clippy::unit_arg)]
            Box::new(move |token| {
                creds::save_api_token_to_keyring(&keychain_service_name, token)
                    .unwrap_or(error!("Couldn't save token to keyring"))
            }),
        ))
    }

    fn api_client_interactive(
        checkvist_base_url: &str,
        keychain_service_name: &str,
        api_token: Option<String>,
    ) -> Result<CheckvistClient> {
        let api_token = match api_token {
            Some(ref token) => token.to_string(),
            None => Self::login_user(checkvist_base_url, keychain_service_name)?,
        };
        let keychain_service_name = keychain_service_name.to_string();
        Ok(CheckvistClient::new(
            checkvist_base_url,
            &api_token,
            #[allow(clippy::unit_arg)]
            Box::new(move |token| {
                creds::save_api_token_to_keyring(&keychain_service_name, token)
                    .unwrap_or(error!("Couldn't save token to keyring"))
            }),
        ))
    }

    /// Returns api_token on success
    fn login_user(checkvist_base_url: &str, keychain_service_name: &str) -> Result<String> {
        let (username, open_api_key) = Self::interact_for_login()?;
        creds::login_user(
            checkvist_base_url,
            keychain_service_name,
            &username,
            &open_api_key,
        )
    }

    // interactions in Context can't rely on a Context! So they're here rather than in Interaction
    pub fn interact_for_login() -> Result<(String, String)> {
        ColourOutput::new(StreamKind::Stderr)
    .append("cvcap is not logged in to Checkvist.", Style::Warning)
    .append("\nPlease enter your Checkvist username and OpenAPI key\nYour username is the email address you log in with\nYour OpenAPI key is available from ", Style::Normal)
    .append("https://checkvist.com/auth/profile", Style::Link)
    .println()
    .expect("Problem printing colour output");

        let username = Input::new()
            .with_prompt("Checkvist Username")
            .interact_text()?;
        let open_api_key = Password::new()
            .with_prompt("Checkvist OpenAPI key")
            .with_confirmation(
                "Confirm OpenAPI key",
                "Please enter your OpenAPI key (available from https://checkvist.com/auth/profile)",
            )
            .interact()?;

        Ok((username, open_api_key))
    }

    /// Returns
    ///   Ok(Some(config)) if user successfully sets up a new config
    ///   Ok(None) if the user cancels
    ///   Err(e) on any error
    pub fn prompt_user_to_set_up_new_config(
        client: &CheckvistClient,
        path: &PathBuf,
    ) -> Result<Option<Config>> {
        if !Confirm::new()
            .with_prompt(
                "cvcap is not yet configured with a default list.\nDo you wish to configure one now?",
            )
            .interact()?
        {
            return Ok(None);
        };

        let Some(selected_list) = interaction::user_select_list(client)? else {
            // user cancels
            return Ok(None);
        };

        let config = Config {
            list_id: selected_list.0,
            list_name: selected_list.1,
            bookmarks: None,
        };
        config.save(path)?;
        ColourOutput::new(StreamKind::Stdout)
            .append(&config.list_name, Style::ListName)
            .append(" is now your default list", Style::Normal)
            .println()?;

        Ok(Some(config))
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

use std::env;

use anyhow::{Context, Result};
use dialoguer::{Input, Password};
use keyring::Entry;

use crate::{ColourOutput, StreamKind, Style};
use cvcap::CheckvistClient;

const NON_DEFAULT_SERVICE_NAME_ENV_KEY: &str = "CVCAP_CREDENTIAL_ID";
const KEYCHAIN_SERVICE_NAME: &str = "cvcap-api-token";

pub fn login_user() -> Result<String> {
    ColourOutput::new(StreamKind::Stderr)
    .append("cvcap is not logged in to Checkvist.", Style::Warning)
    .append("\nPlease enter your Checkvist username and OpenAPI key\nYour username is the email address you log in with\nYour OpenAPI key is available from ", Style::Normal)
    .append("https://checkvist.com/auth/profile", Style::Link)
    .println()
    .expect("Problem printing colour output");

    let token = CheckvistClient::get_token(
        "https://checkvist.com/".into(),
        Input::new()
            .with_prompt("Checkvist Username")
            .interact_text()?,
        Password::new()
            .with_prompt("Checkvist OpenAPI key")
            .with_confirmation(
                "Confirm OpenAPI key",
                "Please enter your OpenAPI key (available from https://checkvist.com/auth/profile)",
            )
            .interact()?,
    )
    .context("Couldn't get token from Checkvist API")?;
    save_api_token_to_keyring(&token)?;

    Ok(token)
}

pub fn save_api_token_to_keyring(token: &str) -> Result<()> {
    let entry = Entry::new(&keychain_service_name(), &whoami::username());
    entry
        .set_password(token)
        .context("Couldn't create keyring entry (for checkvist API token")?;

    Ok(())
}

pub fn get_api_token_from_keyring() -> Option<String> {
    let username = whoami::username();
    Entry::new(&keychain_service_name(), &username)
        .get_password()
        .ok()
}

pub fn delete_api_token() -> Result<(), keyring::Error> {
    let os_username = whoami::username();
    let checkvist_api_token = Entry::new(&keychain_service_name(), &os_username);
    checkvist_api_token.delete_password()
}

fn keychain_service_name() -> String {
    match env::var_os(NON_DEFAULT_SERVICE_NAME_ENV_KEY) {
        Some(name) => name.into_string().expect(
            "Couldn't get a valid credential key from CVCAP_CREDENTIAL_ID environment variable",
        ),
        None => KEYCHAIN_SERVICE_NAME.into(),
    }
}

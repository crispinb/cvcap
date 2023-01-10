use anyhow::{Context, Result};
use dialoguer::{Input, Password};
use keyring::Entry;

use crate::colour_output::{ColourOutput, StreamKind, Style};
use cvapi::CheckvistClient;

pub fn login_user(keychain_service_name: &str) -> Result<String> {
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
    save_api_token_to_keyring(keychain_service_name, &token)?;

    Ok(token)
}

// takes owned params because it needs to be usable in a callback
pub fn save_api_token_to_keyring(keychain_service_name: &str, token: &str) -> Result<()> {
    let entry = Entry::new(keychain_service_name, &whoami::username());
    entry
        .set_password(token)
        .context("Couldn't create keyring entry (for checkvist API token")?;

    Ok(())
}

pub fn get_api_token_from_keyring(keychain_service_name: &str) -> Option<String> {
    let username = whoami::username();
    Entry::new(keychain_service_name, &username)
        .get_password()
        .ok()
}

pub fn delete_api_token(keychain_service_name: &str) -> Result<(), keyring::Error> {
    println!("--> deleting {}", keychain_service_name);
    let os_username = whoami::username();
    let checkvist_api_token = Entry::new(keychain_service_name, &os_username);
    checkvist_api_token.delete_password()
}

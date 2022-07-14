use anyhow::{Context, Result};
use cvcap::CheckvistClient;
use dialoguer::{Input, Password};
use keyring::Entry;

const KEYCHAIN_SERVICE_NAME: &str = "cvcap-api-token";

pub fn login_user() -> Result<String> {
    println!("cvcap is not logged in to Checkvist.\nPlease enter your Checkvist username and OpenAPI key\nYour username is the email address you log in with\nYour OpenAPI key is available from https://checkvist.com/auth/profile");
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
    let entry = Entry::new(KEYCHAIN_SERVICE_NAME, &whoami::username());
    entry
        .set_password(token)
        .context("Couldn't create keyring entry (for checkvist API token")?;

    Ok(())
}

pub fn get_api_token_from_keyring() -> Option<String> {
    let username = whoami::username();
    Entry::new(KEYCHAIN_SERVICE_NAME, &username)
        .get_password()
        .ok()
}

pub fn delete_api_token() -> Result<(), keyring::Error> {
    let os_username = whoami::username();
    let checkvist_api_token = Entry::new(KEYCHAIN_SERVICE_NAME, &os_username);
    checkvist_api_token.delete_password()
}
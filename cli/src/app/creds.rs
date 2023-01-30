use anyhow::{Context, Result};
use keyring::Entry;

use cvapi::CheckvistClient;

// TODO: perhaps move to interactions?
// or maybe ann interact module through which all user interactions are funnelled?
// (and have ALL dialoguer & colour stuff imported there & only htere?)
// Maybe use this as a pattern for other interactive stuff.
/// Returns api_token on success
pub fn login_user(
    checkvist_base_url: &str,
    keychain_service_name: &str,
    username: &str,
    open_api_key: &str,
) -> Result<String> {
    let token = CheckvistClient::get_token(checkvist_base_url, username, open_api_key)
        .context("Couldn't get token from Checkvist API")?;
    save_api_token_to_keyring(keychain_service_name, &token)?;

    Ok(token)
}

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
    let os_username = whoami::username();
    let checkvist_api_token = Entry::new(keychain_service_name, &os_username);
    checkvist_api_token.delete_password()
}

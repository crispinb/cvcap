use keyring::Entry;

// dupes of functions in bin::app::creds.
// May pull those out into a lib instead if we need more here.

pub fn save_api_token_to_keyring(token: &str, service_name: &str) {
    let entry = Entry::new(service_name, &whoami::username());
    entry.set_password(token).expect("couldn't set password");
}

pub fn delete_api_token(service_name: &str) {
    let os_username = whoami::username();
    let checkvist_api_token = Entry::new(service_name, &os_username);
    checkvist_api_token
        .delete_password()
        .map_err(|_| eprintln!("couldn't delete token"))
        .unwrap_or(())
}

pub fn get_api_token_from_keyring(service_name: &str) -> Option<String> {
    let username = whoami::username();
    Entry::new(service_name, &username).get_password().ok()
}

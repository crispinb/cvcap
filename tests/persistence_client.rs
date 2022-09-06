use chrono::prelude::*;
use mockito::{mock, Matcher};
use std::collections::HashMap;
// TODO: reorganise lib & exports
use cvapi::persistent_checkvist_client::PersistentCheckvistClient;
use cvapi::{sqlite_store::SqliteStore, Checklist, CheckvistClient};

// TODO: (for https://github.com/crispinb/cvcap/issues/21)
// * add file-based sqlite
// * add cli 'synclists'
// * use persistence api in cli `-l`

#[test]
// after all, this is really an integration test.
// sqlite store does the storing
// checkvistclient does the getting
fn try_get_list_without_mockall() {
    let lists: Vec<Checklist> = (1..50)
        .map(|i| Checklist {
            id: i,
            name: format!("Checklist {}", i),
            task_count: 10,
            updated_at: Local::now().trunc_subsecs(0).try_into().unwrap(),
        })
        .collect();
    let tasks_json = serde_json::to_string(&lists).unwrap();
    let mock = new_mock_get("/checklists.json", "token", tasks_json);

    let checkvist_client = CheckvistClient::new(mockito::server_url(), "token".into(), |_token| ());
    let sqlite_store = SqliteStore::init().unwrap();
    let client = PersistentCheckvistClient::new(checkvist_client, sqlite_store);
    
    client.sync_lists().unwrap();
    mock.assert();
    let stored_lists = client.fetch_all_lists().unwrap();

    assert_eq!(stored_lists, lists);
}

fn new_mock_get(url: &str, token_to_match: &str, response_body: String) -> mockito::Mock {
    mock("GET", url)
        .match_header("X-Client-Token", token_to_match)
        .with_body(response_body)
        .create()
}

fn new_mock_post(
    url: &str,
    request_body: serde_json::Value,
    response_body: String,
) -> mockito::Mock {
    mock("POST", url)
        .match_header("X-Client-Token", "token")
        .match_body(Matcher::Json(request_body))
        .with_body(response_body)
        .create()
}

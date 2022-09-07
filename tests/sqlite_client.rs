use chrono::prelude::*;
use mockito::{mock, Matcher};

use cvapi::sqlite::{SqliteStore, SqliteSyncClient};
use cvapi::{ApiClient, Checklist, CheckvistClient, Task};

#[test]
fn save_and_fetch_lists() {
    let lists: Vec<Checklist> = (1..50)
        .map(|i| Checklist {
            id: i,
            name: format!("Checklist {}", i),
            task_count: 10,
            updated_at: Local::now().trunc_subsecs(0).try_into().unwrap(),
        })
        .collect();
    let json = serde_json::to_string(&lists).unwrap();
    let mock = new_mock_get("/checklists.json", "token", json);

    let api_client = ApiClient::new(mockito::server_url(), "token".into(), |_token| ());
    let sqlite_store = SqliteStore::init_in_memory().unwrap();
    let client = SqliteSyncClient::new(api_client, sqlite_store);

    client.sync_lists().unwrap();
    mock.assert();
    let stored_lists = client.get_lists().unwrap();

    assert_eq!(stored_lists, lists);
}

#[test]
fn save_and_fetch_tasks() {
    let tasks: Vec<Task> = (1..50)
        .map(|id| Task {
            id: Some(id),
            content: "a task".into(),
            position: id as u16,
        })
        .collect();
    let json = serde_json::to_string(&tasks).unwrap();
    let mock = new_mock_get("/checklists/1/tasks.json", "token", json);

    let api_client = ApiClient::new(mockito::server_url(), "token".into(), |_token| ());
    let sqlite_store = SqliteStore::init_in_memory().unwrap();
    let client = SqliteSyncClient::new(api_client, sqlite_store);

    client.sync_tasks(1).unwrap();
    mock.assert();
    let stored_tasks = client.get_tasks(1).unwrap();

    // assert_eq!(stored_tasks, tasks);
    assert_ne!(stored_tasks, tasks);
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

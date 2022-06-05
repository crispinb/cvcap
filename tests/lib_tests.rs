use checkvistcli::CheckvistClient;
// TODO: refactor tests
//    - do we use one MockServer for all test methods?
//     - if so do we mock all for that or do the Mocks per test function?
//     - if we do have one for all tests, how do we access a 'global'?
#[allow(unused)]
use checkvistcli::{Checklist, CheckvistError, Task};
use mockito::mock;
use std::collections::HashMap;

#[test]
#[should_panic]
fn client_creation_should_panic_with_invalid_url() {
    let _client = CheckvistClient::new("".into(), "token".into());
}

#[test]
fn authentication_failure() {
    let auth_err_msg = "Unauthenticated: no valid authentication data in request";
    let error = HashMap::from([("message", auth_err_msg)]);
    let error_json = serde_json::to_string(&error).unwrap();
    let mock = mock("GET", "/checklists/1.json")
        .match_header("X-Client-Token", "token")
        .with_body(error_json)
        .create();

    let client = CheckvistClient::new(mockito::server_url(), "token".to_string());
    let result = client.get_list(1).unwrap_err();

    mock.assert();
    let returned_err_msg = match result {
        CheckvistError::UnknownError { message } => message,
        _ => String::new(),
    };
    assert_eq!(auth_err_msg, returned_err_msg);
}

#[test]
fn get_list() {
    let expected_list = Checklist {
        id: 1,
        name: "list1".to_string(),
        updated_at: "a date".to_string(),
        task_count: 1,
    };
    let response_json = serde_json::to_string(&expected_list).unwrap();

    let mock = mock("GET", "/checklists/1.json")
        .match_header("X-Client-Token", "token")
        .with_body(response_json)
        .create();

    let client = CheckvistClient::new(mockito::server_url(), "token".into());
    let result = client.get_list(1).unwrap();

    mock.assert();
    assert_eq!(expected_list, result);
}

#[test]
fn get_tasks() {
    let task = Task {
        id: 1,
        content: "content".to_string(),
    };
    let tasks = vec![task];
    let task_json = serde_json::to_string(&tasks).unwrap();

    let mock = mock("GET", "/checklists/1/tasks.json")
        .match_header("X-Client-Token", "token")
        .with_body(task_json)
        .create();

    let client = CheckvistClient::new(mockito::server_url(), "token".into());
    let returned_tasks = client.get_tasks(1).unwrap();

    mock.assert();
    assert_eq!(tasks, returned_tasks);
}

#[test]
fn get_tasks_error() {
    let auth_err_msg = "unauthenticated: no valid authentication data in request";
    let error = HashMap::from([("message", auth_err_msg)]);
    let error_json = serde_json::to_string(&error).unwrap();
    let mock = mock("GET", "/checklists/1/tasks.json")
        .match_header("X-Client-Token", "token")
        .with_body(error_json)
        .create();

    let client = CheckvistClient::new(mockito::server_url(), "token".into());
    let response = client.get_tasks(1).unwrap_err();

    mock.assert();
    let returned_err_msg = match response {
        CheckvistError::UnknownError { message } => message,
        _ => String::new(),
    };
    assert_eq!(auth_err_msg, returned_err_msg);
}

#[test]
fn add_task() {
    unimplemented!("implement this bastard");

    // let mock_server = MockServer::start().await;
    // // 'content' is the only required field
    // let added_task = TempTaskForAdding {
    //     position: 1,
    //     content: "some text".into(),
    // };
    // let returned_task = Task {
    //     id: 1,
    //     content: "some text".into(),
    // };

    // Mock::given(method("POST"))
    //     // TODO: lookup the streamlined header arg
    //     .and(header("X-Client-Token", "token"))
    //     .and(header("Content-Type", "application/json"))
    //     .and(path("/checklists/1/tasks.json"))
    //     .and(body_partial_json(added_task.clone()))
    //     // TODO: add expectation that the task is sent?
    //     .respond_with(ResponseTemplate::new(200).set_body_json(&returned_task))
    //     .expect(1)
    //     .mount(&mock_server)
    //     .await;

    // let client = ChecklistClient::new(mock_server.uri(), "token".to_string());
    // let task = client.add_task(1, &added_task).await.unwrap();

    // assert_eq!(returned_task, task);
}

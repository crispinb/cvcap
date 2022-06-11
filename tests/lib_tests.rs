use checkvistcli::CheckvistClient;
#[allow(unused)]
use checkvistcli::{Checklist, CheckvistError, Task};
use mockito::{mock, Matcher};
use std::collections::HashMap;

#[test]
#[should_panic]
fn client_creation_should_panic_with_invalid_url() {
    let _client = CheckvistClient::new("".into(), "token".into());
}

#[test]
// Checkvist api generates errors as 200 JSON responses with {message: <error message>}
fn authentication_failure_results_in_api_json_error() {
    let auth_err_msg = "Unauthenticated: no valid authentication data in request";
    let error = HashMap::from([("message", auth_err_msg)]);
    let error_json = serde_json::to_string(&error).unwrap();
    let mock = new_mock_get("/checklists/1.json", error_json);

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
fn http_error_results_in_ureq_status_error() {
    let mock = mock("GET", "/checklists/1.json").with_status(404).create();
    let client = CheckvistClient::new(mockito::server_url(), "token".into());
    let returned_error = client.get_list(1).unwrap_err();

    mock.assert();
    if let CheckvistError::NetworkError(ureq::Error::Status(status, _)) = returned_error {
        assert_eq!(status, 404);
    } else {
        panic!("Wrong error type: {:?}", returned_error);
    }
}

#[test]
fn network_error_results_in_ureq_transport_error() {
    let client = CheckvistClient::new("http://localhost".into(), "token".into());
    let returned_error = client.get_tasks(1).unwrap_err();

    match returned_error {
        CheckvistError::NetworkError(ureq::Error::Transport(transport)) => {
            assert_eq!(transport.kind(), ureq::ErrorKind::ConnectionFailed)
        }
        _ => panic!("Wrong error type: {:?}", returned_error),
    }
}

#[test]
fn json_decoding_error() {
    let mock = new_mock_get("/checklists/1.json", "something".into());
    let client = CheckvistClient::new(mockito::server_url(), "token".into());

    let returned_error = client.get_list(1).unwrap_err();

    mock.assert();
    match returned_error {
        CheckvistError::IoError(err) => assert_eq!(err.kind(), std::io::ErrorKind::InvalidData),
        err => panic!("Wrong error type: {:?}", err),
    }
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
    let mock = new_mock_get("/checklists/1.json", response_json);

    let client = CheckvistClient::new(mockito::server_url(), "token".into());
    let result = client.get_list(1).unwrap();

    mock.assert();
    assert_eq!(expected_list, result);
}

#[test]
fn get_tasks() {
    let tasks = vec!(Task {
        id: 1,
        position: 1,
        content: "content".to_string(),
    });
    let task_json = serde_json::to_string(&tasks).unwrap();
    let mock = new_mock_get("/checklists/1/tasks.json", task_json);

    let client = CheckvistClient::new(mockito::server_url(), "token".into());
    let returned_tasks = client.get_tasks(1).unwrap();

    mock.assert();
    assert_eq!(tasks, returned_tasks);
}

#[test]
fn add_task() {
    let task = Task {
        id: 1,
        position: 1,
        content: "some text".into(),
    };
    let body = serde_json::to_string(&task).unwrap();
    let body_json = serde_json::to_value(task.clone()).unwrap();
    let mock = new_mock_post("/checklists/1/tasks.json", body_json, body);

    let client = CheckvistClient::new(mockito::server_url(), "token".into());
    let returned_task = client.add_task(1, task.clone()).unwrap();

    mock.assert();
    assert_eq!(task, returned_task);
}

// Utilities
fn new_mock_get(url: &str, return_body: String) -> mockito::Mock {
    mock("GET", url)
        .match_header("X-Client-Token", "token")
        .with_body(return_body)
        .create()
}

fn new_mock_post(url: &str, send_body: serde_json::Value, return_body: String) -> mockito::Mock {
    mock("POST", url)
        .match_header("X-Client-Token", "token")
        .match_body(Matcher::Json(send_body))
        .with_body(return_body)
        .create()
}

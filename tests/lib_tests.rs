#[allow(unused)]
use cvapi::{CheckvistClient, Checklist, CheckvistError, Task};    
use mockito::{mock, Matcher};
use std::collections::HashMap;

#[test]
#[should_panic]
fn client_creation_should_panic_with_invalid_url() {
    let _client = CheckvistClient::new("".into(), "token".into(), |_token| ());
}

#[test]
fn get_auth_token() {
    let token = "test token";
    let username = "user@test.com";
    let remote_key = "anything";
    let request_body = serde_json::to_value(HashMap::from([
        ("remote_key", remote_key),
        ("username", username),
    ]))
    .unwrap();
    let response_body = serde_json::to_string(&HashMap::from([("token", token)])).unwrap();
    let mock = mock("POST", "/auth/login.json?version=2")
        .match_body(Matcher::Json(request_body))
        .with_body(response_body)
        .create();

    let returned_token =
        CheckvistClient::get_token(mockito::server_url(), username.into(), remote_key.into())
            .unwrap();

    mock.assert();
    assert_eq!(token, returned_token);
}

#[test]
// Checkvist api generates errors as 401 JSON responses with {message: <error message>}
// we can't usefully check the returned body as mockito uses its own body for 401's
fn authentication_failure_results_in_token_refresh_attempt_then_redo() {
    let (old_token, new_token) = ("old token", "token");
    let mock_failed_auth = mock("GET", "/checklists/1.json")
        .match_header("X-Client-Token", old_token)
        .with_status(401)
        .create();
    let refresh_response_body =
        serde_json::to_string(&HashMap::from([("token", new_token)])).unwrap();
    let mock_refresh = mock("POST", "/auth/refresh_token.json?version=2")
        .with_body(refresh_response_body)
        .create();
    let list_name = "list1";
    let expected_list = Checklist {
        id: 1,
        name: list_name.into(),
        updated_at: "a date".to_string(),
        task_count: 1,
    };
    let response_json = serde_json::to_string(&expected_list).unwrap();
    let mock_success_auth = new_mock_get("/checklists/1.json", new_token, response_json);

    // TODO: could change the new() callback to be a closure so it can capture 'new_token' rather than use literal here
    let client = CheckvistClient::new(mockito::server_url(), old_token.into(), |token| {
        assert_eq!(
            token, "token",
            "token refresh callback received wrong token value"
        )
    });
    let result = client.get_list(1);

    mock_failed_auth.assert();
    mock_refresh.assert();
    mock_success_auth.assert();
    assert_eq!(result.unwrap().name, list_name.to_string());
}

#[test]
fn refresh_failure_results_in_401_error() {
    let mock_failed_auth = mock("GET", "/checklists/1.json").with_status(401).create();
    let mock_refresh = mock("POST", "/auth/refresh_token.json?version=2")
        .with_status(401)
        .create();

    let client = CheckvistClient::new(mockito::server_url(), String::from("token"), |_token| ());
    let returned_error = client.get_list(1).unwrap_err();

    mock_failed_auth.assert();
    mock_refresh.assert();

    assert!(
        std::matches!(returned_error, CheckvistError::TokenRefreshFailedError),
        "Failed to refresh token: {}",
        returned_error
    );
}

#[test]
fn http_error_results_in_ureq_status_error() {
    let mock = mock("GET", "/checklists/1.json").with_status(404).create();
    let client = CheckvistClient::new(mockito::server_url(), "token".into(), |_token| ());
    let returned_error = client.get_list(1).unwrap_err();

    mock.assert();
    assert!(
        std::matches!(
            returned_error,
            CheckvistError::NetworkError(ureq::Error::Status(404, _response))
        ),
        "get_list() returned an error, but the status code wasn't 404"
    );
}

#[test]
fn network_error_results_in_ureq_transport_error() {
    let client = CheckvistClient::new("http://localhost".into(), "token".into(), |_token| ());
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
    let mock = new_mock_get("/checklists/1.json", "token", "something".into());
    let client = CheckvistClient::new(mockito::server_url(), "token".into(), |_token| ());

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
    let mock = new_mock_get("/checklists/1.json", "token", response_json);

    let client = CheckvistClient::new(mockito::server_url(), "token".into(), |_token| ());
    let result = client.get_list(1).unwrap();

    mock.assert();
    assert_eq!(expected_list, result);
}

#[test]
fn add_list() {
    let new_list = "test list";
    let expected_list = Checklist {
        id: 1,
        name: new_list.into(),
        updated_at: "a date".to_string(),
        task_count: 0,
    };

    let request_body = serde_json::to_value(HashMap::from([("name", new_list)])).unwrap();
    let response_json = serde_json::to_string(&expected_list).unwrap();
    let mock = new_mock_post("/checklists.json", request_body, response_json);

    let client = CheckvistClient::new(mockito::server_url(), "token".into(), |_token| ());
    let result = client.add_list(new_list).unwrap();

    mock.assert();
    assert_eq!(result, expected_list);
}

#[test]
fn get_tasks() {
    let tasks = vec![Task {
        id: Some(1),
        position: 1,
        content: "content".to_string(),
    }];
    let task_json = serde_json::to_string(&tasks).unwrap();
    let mock = new_mock_get("/checklists/1/tasks.json", "token", task_json);

    let client = CheckvistClient::new(mockito::server_url(), "token".into(), |_token| ());
    let returned_tasks = client.get_tasks(1).unwrap();

    mock.assert();
    assert_eq!(tasks, returned_tasks);
}

#[test]
fn add_task() {
    let task = Task {
        id: Some(1),
        position: 1,
        content: "some text".into(),
    };
    let response_body = serde_json::to_string(&task).unwrap();
    let request_body = serde_json::to_value(task.clone()).unwrap();
    let mock = new_mock_post("/checklists/1/tasks.json", request_body, response_body);

    let client = CheckvistClient::new(mockito::server_url(), "token".into(), |_token| ());
    let returned_task = client.add_task(1, &task).unwrap();

    mock.assert();
    assert_eq!(task, returned_task);
}

#[test]
// curl --json '{"old_token": ""}'  "https://checkvist.com/auth/refresh_token.json?version=2"
fn refresh_auth_token() {
    let old_token = "token";
    let new_token = "new token";
    let request_body = serde_json::to_value(&HashMap::from([("old_token", old_token)])).unwrap();
    let response_body = serde_json::to_string(&HashMap::from([("token", new_token)])).unwrap();
    let mock = mock("POST", "/auth/refresh_token.json?version=2")
        .match_body(Matcher::Json(request_body))
        .with_body(response_body)
        .create();

    let client = CheckvistClient::new(mockito::server_url(), "token".to_string(), |t| {
        assert_eq!(
            t, "new token",
            "Token refresh closure received unexpected token"
        )
    });

    client.refresh_token().unwrap();

    mock.assert();
}

#[test]
fn refresh_auth_token_error_on_failure() {
    let request_body = serde_json::to_value(HashMap::from([("old_token", "token")])).unwrap();
    let mock = mock("POST", "/auth/refresh_token.json?version=2")
        .match_body(Matcher::Json(request_body))
        .with_status(401)
        .create();
    let client = CheckvistClient::new(mockito::server_url(), "token".to_string(), |_t| ());

    let err = client.refresh_token().unwrap_err();

    mock.assert();
    assert!(std::matches!(err, CheckvistError::TokenRefreshFailedError));
}

// Utilities
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

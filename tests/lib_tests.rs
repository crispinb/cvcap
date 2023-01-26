#[allow(unused)]
use std::collections::HashMap;

use serde_json::json;
use wiremock::matchers::{body_partial_json, header, method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

use cvapi::{Checklist, CheckvistClient, CheckvistError, Task};

#[test]
#[should_panic]
fn client_creation_should_panic_with_invalid_url() {
    let _client = CheckvistClient::new("".into(), "token".into(), Box::new(|_token| ()));
}

#[tokio::test]
async fn get_auth_token() {
    let token = "test token";
    let username = "user@test.com";
    let remote_key = "anything";
    let request_body = json!(HashMap::from([
        ("remote_key", remote_key),
        ("username", username),
    ]));
    let response_body = &HashMap::from([("token", token)]);
    let mock_server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/auth/login.json"))
        .and(query_param("version", "2"))
        .and(body_partial_json(request_body))
        .respond_with(ResponseTemplate::new(200).set_body_json(response_body))
        .mount(&mock_server)
        .await;

    let returned_token =
        CheckvistClient::get_token(mock_server.uri(), username.into(), remote_key.into()).unwrap();

    assert_eq!(token, returned_token);
}

#[tokio::test]
async fn authentication_failure_results_in_token_refresh_attempt_then_redo() {
    let (old_token, new_token) = ("old token", "token");
    let token_refresh_response = HashMap::from([("token", new_token)]);
    let list_name = "list1";
    let expected_list = Checklist {
        id: 1,
        name: list_name.into(),
        updated_at: "a date".to_string(),
        task_count: 1,
    };
    let mock_server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/checklists/1.json"))
        .and(header("X-Client-Token", old_token))
        .respond_with(ResponseTemplate::new(401))
        .expect(1)
        .mount(&mock_server)
        .await;
    Mock::given(method("POST"))
        .and(path("/auth/refresh_token.json"))
        .and(query_param("version", "2"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!(token_refresh_response)))
        .mount(&mock_server)
        .await;
    Mock::given(method("GET"))
        .and(path("/checklists/1.json"))
        .and(header("X-Client-Token", new_token))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!(expected_list)))
        .expect(1)
        .mount(&mock_server)
        .await;

    let client = CheckvistClient::new(
        &mock_server.uri(),
        old_token.into(),
        Box::new(|token| {
            assert_eq!(
                token, "token",
                "token refresh callback received wrong token value"
            )
        }),
    );
    let result = client.get_list(1);

    assert_eq!(result.unwrap().name, list_name.to_string());
}

#[tokio::test]
async fn refresh_failure_results_in_401_error() {
    let mock_server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/checklists/1.json"))
        .respond_with(ResponseTemplate::new(401))
        .expect(1)
        .mount(&mock_server)
        .await;
    Mock::given(method("POST"))
        .and(path("/auth/refresh_token.json"))
        .respond_with(ResponseTemplate::new(401))
        .expect(1)
        .mount(&mock_server)
        .await;
    let client = CheckvistClient::new(
        &mock_server.uri(),
        &String::from("token"),
        Box::new(|_token| ()),
    );

    let returned_error = client.get_list(1).unwrap_err();

    assert!(
        std::matches!(returned_error, CheckvistError::TokenRefreshFailedError),
        "Failed to refresh token: {}",
        returned_error
    );
}

#[tokio::test]
async fn http_error_results_in_ureq_status_error() {
    let mock_server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/checklists/1.json"))
        .respond_with(ResponseTemplate::new(404))
        .mount(&mock_server)
        .await;
    let client = CheckvistClient::new(&mock_server.uri(), "token".into(), Box::new(|_token| ()));

    let returned_error = client.get_list(1).unwrap_err();

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
    let client = CheckvistClient::new(
        "http://localhost".into(),
        "token".into(),
        Box::new(|_token| ()),
    );
    let returned_error = client.get_tasks(1).unwrap_err();

    match returned_error {
        CheckvistError::NetworkError(ureq::Error::Transport(transport)) => {
            assert_eq!(transport.kind(), ureq::ErrorKind::ConnectionFailed)
        }
        _ => panic!("Wrong error type: {:?}", returned_error),
    }
}

#[tokio::test]
async fn get_json_decoding_error_from_server_gibberish() {
    let mock_server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/checklists/1.json"))
        .respond_with(ResponseTemplate::new(200).set_body_string("any old gibberish"))
        .expect(1)
        .mount(&mock_server)
        .await;
    let client = CheckvistClient::new(&mock_server.uri(), "token".into(), Box::new(|_token| ()));

    let returned_error = client.get_list(1).unwrap_err();

    match returned_error {
        CheckvistError::IoError(err) => assert_eq!(err.kind(), std::io::ErrorKind::InvalidData),
        err => panic!("Wrong error type: {:?}", err),
    }
}

#[tokio::test]
async fn get_list() {
    let expected = Checklist {
        id: 1,
        name: "list1".to_string(),
        updated_at: "a date".to_string(),
        task_count: 1,
    };
    let mock_server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/checklists/1.json"))
        .and(header("X-Client-Token", "token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!(expected)))
        .mount(&mock_server)
        .await;

    let client = CheckvistClient::new(&mock_server.uri(), "token".into(), Box::new(|_token| ()));
    let result = client.get_list(1).unwrap();

    assert_eq!(expected, result);
}

#[tokio::test]
async fn add_list() {
    let new_list = "test list";
    let expected = Checklist {
        id: 1,
        name: new_list.into(),
        updated_at: "a date".to_string(),
        task_count: 0,
    };

    let request_body = HashMap::from([("name", new_list)]);
    let mock_server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/checklists.json"))
        .and(header("X-Client-Token", "token"))
        .and(body_partial_json(json!(request_body)))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!(expected)))
        .mount(&mock_server)
        .await;
    let client = CheckvistClient::new(&mock_server.uri(), "token".into(), Box::new(|_token| ()));

    let result = client.add_list(new_list).unwrap();

    assert_eq!(result, expected);
}

#[tokio::test]
async fn get_tasks() {
    let tasks = vec![Task {
        id: Some(1),
        position: 1,
        content: "content".to_string(),
        parent_id: None,
    }];
    let mock_server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/checklists/1/tasks.json"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!(tasks)))
        .expect(1)
        .mount(&mock_server)
        .await;

    let client = CheckvistClient::new(&mock_server.uri(), "token".into(), Box::new(|_token| ()));
    let returned_tasks = client.get_tasks(1).unwrap();

    assert_eq!(tasks, returned_tasks);
}

#[tokio::test]
async fn add_task_to_list() {
    let task = Task {
        id: Some(1),
        position: 1,
        content: "some text".into(),
        parent_id: None,
    };
    let mock_server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/checklists/1/tasks.json"))
        .and(body_partial_json(json!(task)))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!(task)))
        .expect(1)
        .mount(&mock_server)
        .await;
    let client = CheckvistClient::new(
        &mock_server.uri(),
        "token".into(),
        Box::new(|_token| ()),
    );

    let returned_task = client.add_task(1, &task).unwrap();

    assert_eq!(task, returned_task);
}

#[tokio::test]
// Checkvist's own API errors are in the format:
// {"message": "error detail"}
// Examples:
//   {"message":"Invalid parent_id: 5885799"}
//   {"message":"The list doesn't exist or is not available to you"}
async fn add_task_checkvist_api_error() {
    let task = Task {
        id: Some(1),
        position: 1,
        content: "some text".into(),
        parent_id: Some(2),
    };
    let mock_server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/checklists/1/tasks.json"))
        .and(body_partial_json(json!(task)))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!(HashMap::from([( "message", "error detail" )]))))
        .expect(1)
        .mount(&mock_server)
        .await;
    
    let client = CheckvistClient::new(
        &mock_server.uri(),
        "token".into(),
        Box::new(|_token| ()),
    );

    let returned_task = client.add_task(1, &task).unwrap_err();
dbg!(&returned_task);
    assert!(
        matches!(returned_task, CheckvistError::UnknownError{message: msg} if msg == "error detail")
    );
}


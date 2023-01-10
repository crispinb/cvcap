use std::collections::HashMap;
use std::path::PathBuf;

use assert_cmd::Command;
use predicates::prelude::*;
use temp_dir::TempDir;
use uuid::Uuid;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

use cvapi::Task;
use cvcap::app::{
    creds,
    config::Config,
    context::{self, CUSTOM_CONFIG_FILE_PATH_ENV_KEY, CUSTOM_SERVICE_URL_KEY},
};

/// These tests are pretty thin - mainly just that the UI requests & reports correctly
/// given the specified args. Underlying functionality is in unit & lib tests
/// NOTE: cvcap isolation from the user's environment is controlled only by env vars.
///       The appropriate ones must be set, or we risk overwriting the users's config file!
/// NB: Some of the more interactive features (eg `-l`) aren't tested at all

#[tokio::test]
async fn run_without_args_shows_help() {
    let (mut cmd, _test_config) = configure_command(HttpResponse::Ok, false, true).await;
    cmd.assert()
        .stderr(predicate::str::contains("USAGE:"))
        .failure();
}

#[tokio::test]
async fn adds_task_without_subcommand() {
    let (mut cmd, _test_config) = configure_command(HttpResponse::Ok, true, true).await;
    cmd.arg("status")
        .assert()
        .stdout(predicate::str::contains("Test List").count(1))
        .stdout(predicate::str::contains("✅"))
        .success();
}

#[tokio::test]
async fn status_reports_default_list_not_configured() {
    let (mut cmd, _test_config) = configure_command(HttpResponse::Ok, false, true).await;
    cmd.arg("status")
        .env(
            context::CUSTOM_CONFIG_FILE_PATH_ENV_KEY,
            "this should not matter!",
        )
        .assert()
        .stdout(predicate::str::is_match("default list:\\s+❌").expect("bad regex").count(1))
        .success();
}

#[tokio::test]
async fn status_reports_user_not_logged_in() {
    let (mut cmd, _test_config) = configure_command(HttpResponse::Ok, true, false).await;

    cmd.arg("status")
        .assert()
        .stdout(predicate::str::is_match("logged in to Checkvist:\\s+❌").expect("bad regex").count(1))
        .success();
}
 #[tokio::test]
async fn status_reports_presence_of_bookmarks() {
    let (mut cmd, _testconfig) = configure_command(HttpResponse::Ok, true, true).await;

    cmd.arg("status")
        .assert()
        .stdout(predicate::str::contains("✅").count(2))
        .success();
}

#[tokio::test]
async fn logout_subcommand_when_not_logged_in_succeeds_with_message() {
    let (mut cmd, _test_config) = configure_command(HttpResponse::Ok, true, false).await;

    cmd.arg("logout")
        .assert()
        .stdout(predicates::str::contains("already logged out"))
        .success();
}

#[tokio::test]
async fn shows_must_login_message_when_token_refresh_fails() {
    let (mut cmd, test_config) = configure_command(HttpResponse::Unauthorised, true, true).await;

    cmd.arg("task to add")
        .arg("-v")
        .assert()
        .stderr(predicate::str::contains(
            "You have been logged out of the Checkvist API",
        ))
        .failure();

    println!(
        "requests: {:?}",
        test_config.mock_server.received_requests().await
    );
}

#[tokio::test]
async fn add_task_from_stdin() {
    let (mut cmd, _test_config) = configure_command(HttpResponse::Ok, true, true).await;

    cmd.arg("add")
        .arg("-s")
        .pipe_stdin("tests/data/task.md")
        .unwrap()
        .assert()
        .stdout(predicate::str::contains("This is a test task"))
        .success();
}

#[tokio::test]
async fn add_task_with_list_bookmark() {
    let (mut cmd, _test_config) = configure_command(HttpResponse::Ok, true, true).await;
    cmd.arg("add")
        .arg("task_with_list_bookmark")
        .arg("-b")
        .arg("list1_bookmark")
        .assert()
        .stdout(predicate::str::contains("Task added").count(1))
        // command output should include bookmark name
        .stdout(predicate::str::contains("list1_bookmark").count(1))
        .success();
}

#[tokio::test]
async fn add_task_with_task_bookmark() {
    let (mut cmd, _test_config) = configure_command(HttpResponse::Ok, true, true).await;
    cmd.arg("add")
        .arg("task_with_task_bookmark")
        .arg("-b")
        .arg("task1_bookmark")
        .assert()
        .stdout(predicate::str::contains("Task added").count(1))
        // command output should include bookmark name
        .stdout(predicate::str::contains("task1_bookmark").count(1))
        .success();
}

#[tokio::test]
async fn logout_subcommand_deletes_token() {
    let (mut cmd, _test_config) = configure_command(HttpResponse::Ok, true, true).await;
    cmd.arg("logout")
        .assert()
        .stdout(predicate::str::contains("logged out"))
        .success();
}

#[tokio::test]
async fn default_add_and_options_conflict() {
    args_should_conflict(vec!["a task", "-l"]).await;
}

#[tokio::test]
async fn s_and_c_flags_conflict() {
    args_should_conflict(vec!["add", "-c", "-s"]).await;
}

async fn args_should_conflict(args: Vec<&str>) {
    let (mut cmd, _test_config) = configure_command(HttpResponse::Ok, false, false).await;
    cmd.args(args).assert().failure();
}

#[tokio::test]
async fn c_flag_conflicts_with_content_arg() {
    args_should_conflict(vec!["add", "content", "-c"]).await;
}

#[tokio::test]
async fn s_flag_conflicts_with_content_arg() {
    args_should_conflict(vec!["add", "content", "-s"]).await;
}

///////////////////////////////////////////////////////////////////////
// TEST HELPERS

/// hold on to resources that need cleaning up
struct TestConfig {
    _temp_dir: TempDir,
    mock_server: MockServer,
    logged_in: bool,
    pub keychain_service_name: String,
}

// clear up test resources
impl std::ops::Drop for TestConfig {
    fn drop(&mut self) {
        if self.logged_in{ 
        creds::delete_api_token(&self.keychain_service_name)
            // NB: this isn't an error where the token was intentionally deleted during
            // the test, eg. for logouts, failed token refreshes, etc
            // So we don't want to unwrap and cause a panic here
            .map_err(|e| println!("Didn't delete API token: {}", e))
            .ok();
        }
    }
}

enum HttpResponse {
    Ok,
    Unauthorised,
}

async fn configure_command(
    response: HttpResponse,
    config_file_exists: bool,
    logged_in: bool,
) -> (Command, TestConfig) {
    let temp_dir = TempDir::new().unwrap();
    let config_path = if config_file_exists {
        let config_path = temp_dir.child("temp.toml");
        let config = config();
        config.write_to_new_file(&config_path).unwrap();
        config_path
    } else {
        PathBuf::from("nonexistent_file")
    };
    let keychain_service_name = if logged_in {
        let service_name = random_service_name();
        creds::save_api_token_to_keyring(&service_name,&random_name("api-token")).unwrap();
        service_name
    } else {
        "cvcap-cli_integration_tests-nonexistent-keyring-service-name".into()
    };

    let task = task();
    let mock_server = mock_server(response, &task).await;

    let mut cmd = Command::cargo_bin("cvcap").unwrap();
    cmd.env(CUSTOM_SERVICE_URL_KEY, &mock_server.uri())
        .env(CUSTOM_CONFIG_FILE_PATH_ENV_KEY, config_path)
        .env(context::CUSTOM_SERVICE_NAME_ENV_KEY, &keychain_service_name);

    (
        cmd,
        TestConfig {
            _temp_dir: temp_dir,
            mock_server,
            logged_in,
            keychain_service_name,
        },
    )
}

async fn mock_server(response: HttpResponse, task: &Task) -> MockServer {
    // create a mock server
    let mock_server = MockServer::start().await;

    let response = match response {
        HttpResponse::Ok => ResponseTemplate::new(200).set_body_json(task),
        HttpResponse::Unauthorised => ResponseTemplate::new(401),
    };

    Mock::given(method("POST"))
        .and(path("/checklists/1/tasks.json"))
        .respond_with(response)
        .mount(&mock_server)
        .await;

    mock_server
}

fn task() -> Task {
    Task {
        id: Some(1),
        position: 1,
        content: "some text".into(),
        parent_id: Some(2),
    }
}

fn config() -> Config {
    Config {
        list_id: 1,
        list_name: "Test List".into(),
        bookmarks: Some(HashMap::from([
            (
                "list1_bookmark".into(),
                "https://beta.checkvist.com/checklists/1".into(),
            ),
            (
                "task1_bookmark".into(),
                "https://beta.checkvist.com/checklists/1/tasks/1".into(),
            ),
        ])),
    }
}

fn random_service_name() -> String {
    random_name("cvcap-test-service")
}

fn random_name(prefix: &str) -> String {
    format!("{}-{}", prefix, Uuid::new_v4())
}

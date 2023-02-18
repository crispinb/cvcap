use std::path::PathBuf;

use assert_cmd::Command;
use copypasta::{ClipboardContext, ClipboardProvider};
use predicates::prelude::*;
use serial_test::serial;
use temp_dir::TempDir;
use uuid::Uuid;
use wiremock::matchers::{method, path, path_regex};
use wiremock::{Mock, MockServer, ResponseTemplate};

use cvapi::{Checklist, CheckvistLocation, Task};
use cvcap::{
    bookmark::Bookmark,
    config::Config,
    context::{self, CUSTOM_CONFIG_FILE_PATH_ENV_KEY, CUSTOM_SERVICE_URL_KEY},
    creds,
};

/// These tests are pretty thin - mainly just that the UI requests & reports correctly
/// given the specified args. Underlying functionality is in unit & lib tests
/// NOTE: cvcap isolation from the user's environment is controlled only by env vars.
///       The appropriate ones must be set, or we risk overwriting the users's config file!
/// Some of the more interactive features (eg `-l`) aren't tested at all
/// ## Untested interactive features
/// * add-bookmark:
///     * y/n prompt if bookmark already exists

#[tokio::test]
async fn run_without_args_shows_help() {
    let (mut cmd, _test_config) = configure_command(None, false, true).await;
    cmd.assert()
        .stderr(predicate::str::contains("USAGE:"))
        .failure();
}

#[tokio::test]
async fn adds_task_without_subcommand() {
    let (mut cmd, _test_config) = configure_command(None, true, true).await;
    cmd.arg("test task")
        .assert()
        .stdout(predicate::str::contains("test task").count(1))
        .success();
}

#[tokio::test]
async fn status_reports_default_list_not_configured() {
    let (mut cmd, _test_config) = configure_command(None, false, true).await;
    cmd.arg("status")
        .env(
            context::CUSTOM_CONFIG_FILE_PATH_ENV_KEY,
            "this should not matter!",
        )
        .assert()
        .stdout(
            predicate::str::is_match("default list:\\s+❌")
                .expect("bad regex")
                .count(1),
        )
        .success();
}

#[tokio::test]
async fn status_reports_user_not_logged_in() {
    let (mut cmd, _test_config) = configure_command(None, true, false).await;

    cmd.arg("status")
        .assert()
        .stdout(
            predicate::str::is_match("logged in to Checkvist:\\s+❌")
                .expect("bad regex")
                .count(1),
        )
        .success();
}
#[tokio::test]
async fn status_reports_presence_of_bookmarks() {
    let (mut cmd, _testconfig) = configure_command(None, true, true).await;

    cmd.arg("status")
        .assert()
        .stdout(predicate::str::contains("✅").count(2))
        .success();
}

// FIX: see https://github.com/crispinb/cvcap/issues/29
#[tokio::test]
async fn logout_subcommand_when_not_logged_in_succeeds_with_message() {
    let (mut cmd, _test_config) = configure_command(None, true, false).await;

    cmd.arg("logout")
        .assert()
        .stdout(predicates::str::contains("already logged out"))
        .success();
}

#[tokio::test]
async fn shows_must_login_message_when_token_refresh_fails() {
    let (mut cmd, test_config) =
        configure_command(Some(HttpResponse::Unauthorised), true, true).await;

    cmd.arg("task to add")
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
    let (mut cmd, _test_config) = configure_command(None, true, true).await;

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
    let (mut cmd, _test_config) = configure_command(None, true, true).await;
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
    let (mut cmd, _test_config) = configure_command(None, true, true).await;
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
    let (mut cmd, _test_config) = configure_command(None, true, true).await;
    cmd.arg("logout")
        .assert()
        .stdout(predicate::str::contains("logged out"))
        .success();
}

// NB: all tests using the ClipboardContext must be `#[serial]`
#[tokio::test]
#[serial]
async fn add_bookmark() {
    let (mut cmd, _test_config) = configure_command(None, true, true).await;
    let cliptext = "https://checkvist.com/checklists/3";
    let mut clip_ctx = ClipboardContext::new().unwrap();
    clip_ctx.set_contents(cliptext.into()).unwrap();

    cmd.arg("add-bookmark")
        .arg("bookmark_name1234")
        .assert()
        .stdout(predicate::str::contains("Bookmark Added").count(1))
        .success();
}

#[tokio::test]
#[serial]
async fn add_bookmark_with_invalid_list_id_fails() {
    let (mut cmd, _test_config) = configure_command(None, true, true).await;
    let list_id = 10;
    let cliptext = format!("https://checkvist.com/checklists/{}", list_id);
    let mut clip_ctx = ClipboardContext::new().unwrap();
    clip_ctx.set_contents(cliptext.into()).unwrap();

    cmd.arg("add-bookmark")
        .arg("test_bookmark")
        .assert()
        .failure();
}

#[tokio::test]
#[serial]
async fn add_bookmark_with_invalid_parent_task_id_fails() {
    let (mut cmd, _test_config) = configure_command(None, true, true).await;
    let list_id = 8;
    let parent_task_id = 10;
    let cliptext = format!(
        "https://checkvist.com/checklists/{}/tasks/{}.json",
        list_id, parent_task_id
    );
    let mut clip_ctx = ClipboardContext::new().unwrap();
    clip_ctx.set_contents(cliptext.into()).unwrap();

    cmd.arg("add-bookmark")
        .arg("test_bookmark")
        .assert()
        .failure();
}

#[tokio::test]
#[serial]
async fn add_bookmark_with_q_succeeds_silently() {
    let (mut cmd, _test_config) = configure_command(None, true, true).await;
    let cliptext = "https://checkvist.com/checklists/3";
    let mut clip_ctx = ClipboardContext::new().unwrap();
    clip_ctx.set_contents(cliptext.into()).unwrap();

    cmd.arg("add-bookmark")
        .arg("-q")
        .arg("bookmark_name1234")
        .assert()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::is_empty())
        .success();
}

#[tokio::test]
#[serial]
async fn add_bookmark_no_config_q_fails_silently() {
    let (mut cmd, _test_config) = configure_command(None, false, true).await;

    cmd.arg("add-bookmark")
        .arg("-q")
        .arg("bookmark")
        .assert()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::is_empty())
        .failure();
}

#[tokio::test]
#[serial]
async fn add_bookmark_logged_out_q_fails_silently() {
    let (mut cmd, _test_config) = configure_command(None, true, false).await;

    cmd.arg("add-bookmark")
        .arg("-q")
        .arg("bookmark")
        .assert()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::is_empty())
        .failure();
}

#[tokio::test]
#[serial]
async fn add_bookmark_with_invalid_list_id_q_fails_silently() {
    let (mut cmd, _test_config) = configure_command(None, true, true).await;
    let list_id = 10;
    let cliptext = format!("https://checkvist.com/checklists/{}", list_id);
    let mut clip_ctx = ClipboardContext::new().unwrap();
    clip_ctx.set_contents(cliptext.into()).unwrap();

    cmd.arg("add-bookmark")
        .arg("test_bookmark")
        .arg("-q")
        .assert()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::is_empty())
        .failure();
}

#[tokio::test]
#[serial]
async fn add_duplicate_bookmark_q_fails_silently() {
    let (mut cmd, _test_config) = configure_command(None, true, true).await;
    // dupes the bookmark set in configure_command
    let cliptext = "https://checkvist.com/checklists/1";
    let mut clip_ctx = ClipboardContext::new().unwrap();
    clip_ctx.set_contents(cliptext.into()).unwrap();

    cmd.arg("add-bookmark")
        .arg("bm1")
        .arg("-q")
        .assert()
        .stderr(predicate::str::is_empty())
        .failure();
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
    let (mut cmd, _test_config) = configure_command(None, false, false).await;
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
    config_path: PathBuf,
}

// clear up test resources
impl std::ops::Drop for TestConfig {
    fn drop(&mut self) {
        if self.logged_in {
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
    Unauthorised,
}

/// Creates these test resources:
/// - configuration env vars for service url, path to config file (if config_file is true),
///   and OS-specific  keychain service name (for api token storage).
/// - config file if 'config_file_exists', with:
///     - 1 each list and task bookmark with list/parent_task_id 1
/// - mock server responding thusly:
///      Auth successs/failure is determined by `response`.
///      Then success/failure for specific responses is determined by the list/task_id
///      args sent to CheckvistClient methods.
///     Successes:
///      - GET request for list ids 1-9
///      - GET request for tasks from list 1-9
///      Failures:
///      - GET request 403 invalid list for any other list
///      - GET request 403 invalid parent task id for any other task
///      - POST to add a task. Returns response & payload from args
async fn configure_command(
    response: Option<HttpResponse>,
    config_file_exists: bool,
    logged_in: bool,
) -> (Command, TestConfig) {
    let temp_dir = TempDir::new().unwrap();
    let config_path = if config_file_exists {
        let config_path = temp_dir.child("temp.toml");
        let config = config();
        config.save(&config_path).unwrap();
        config_path
    } else {
        PathBuf::from("nonexistent_file")
    };
    let keychain_service_name = if logged_in {
        let service_name = random_service_name();
        creds::save_api_token_to_keyring(&service_name, &random_name("api-token")).unwrap();
        service_name
    } else {
        "cvcap-cli_integration_tests-nonexistent-keyring-service-name".into()
    };

    let mock_server = mock_server(response).await;

    let mut cmd = Command::cargo_bin("cvcap").unwrap();
    cmd.env(CUSTOM_SERVICE_URL_KEY, &mock_server.uri())
        .env(CUSTOM_CONFIG_FILE_PATH_ENV_KEY, &config_path)
        .env(context::CUSTOM_SERVICE_NAME_ENV_KEY, &keychain_service_name);

    (
        cmd,
        TestConfig {
            _temp_dir: temp_dir,
            mock_server,
            logged_in,
            keychain_service_name,
            config_path,
        },
    )
}

async fn mock_server(response: Option<HttpResponse>) -> MockServer {
    // create a mock server
    let mock_server = MockServer::start().await;

    // response to use when neither the caller has specified one,
    // nor is there a unique one added in the mocks
    let default_response = match response {
        None => ResponseTemplate::new(200),
        Some(HttpResponse::Unauthorised) => ResponseTemplate::new(401),
    };

    let list = list();
    let task = task();
    let tasks = vec![&task];

    // wiremock docs don't make clear how matchers interact
    // It seems exact path matches beat regexes, and that
    // order only matters for an exact clash (first wins)
    Mock::given(method("GET"))
        .and(path_regex(r#"/checklists/[1-9].json"#))
        .respond_with(default_response.clone().set_body_json(list))
        .mount(&mock_server)
        .await;

    let err_msg = r#"{"message":"The list doesn't exist or is not available to you"}"#;
    Mock::given(method("GET"))
        .and(path_regex(r#"/checklists/\d+\.json"#))
        .respond_with(ResponseTemplate::new(403).set_body_string(err_msg))
        .mount(&mock_server)
        .await;

    Mock::given(method("POST"))
        .and(path_regex("/checklists/[1-9]/tasks.json"))
        .respond_with(default_response.clone().set_body_json(&task))
        .mount(&mock_server)
        .await;

    Mock::given(method("GET"))
        .and(path("/checklists/[1-9]/tasks.json"))
        .respond_with(default_response.clone().set_body_json(&tasks))
        .mount(&mock_server)
        .await;

    // does the err parent id really matter for testing here?
    let task_err_msg = r#"{"message":"Invalid parent_id: 7"}"#;
    Mock::given(method("GET"))
        .and(path_regex(r#"/checklists/\d+/tasks/\d+\.json"#))
        .respond_with(ResponseTemplate::new(400).set_body_string(task_err_msg))
        .mount(&mock_server)
        .await;

    mock_server
}

fn list() -> Checklist {
    Checklist {
        id: 1,
        name: "Test List".into(),
        updated_at: "".into(),
        task_count: 1,
    }
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
    let list_location = CheckvistLocation {
        list_id: 1,
        parent_task_id: None,
    };
    let task_location = CheckvistLocation {
        list_id: 1,
        parent_task_id: Some(1),
    };
    Config {
        list_id: 1,
        list_name: "Test List".into(),
        bookmarks: Some(vec![
            Bookmark {
                name: "list1_bookmark".into(),
                location: list_location,
            },
            Bookmark {
                name: "task1_bookmark".into(),
                location: task_location,
            },
        ]),
    }
}

fn random_service_name() -> String {
    random_name("cvcap-test-service")
}

fn random_name(prefix: &str) -> String {
    format!("{}-{}", prefix, Uuid::new_v4())
}

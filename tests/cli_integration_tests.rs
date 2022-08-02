mod test_config;
mod test_creds;
use cvcap::{Checklist, CheckvistClient};
use predicates::prelude::*;
use std::path;
use std::sync::Once;
use temp_dir::TempDir;
use test_config::TestCvcapRunConfig;
use uuid::Uuid;


// TODO: remove added tasks after tests
// TODO: how to test interactive login, '-l' flag etc

static CREATE_TEST_LIST: Once = Once::new();
static mut TEST_LIST: Checklist = Checklist {
    id: 0,
    name: String::new(),
    updated_at: String::new(),
    task_count: 0,
};
const API_KEY_ENV: &str = "CVCAP_API_TOKEN";
const TEST_LIST_NAME: &str = "cvcap cli integration tests";

#[test]
#[ignore = "cvcap bin run (slow)"]
fn run_without_args_shows_help() {
    let mut cmd = assert_cmd::Command::cargo_bin("cvcap").unwrap();
    cmd.assert()
        .stderr(predicate::str::contains("USAGE:"))
        .failure();
}

#[test]
#[ignore = "cvcap bin run (slow)"]
fn adds_task_without_subcommand() {
    let mut config = TestConfig::new(true, true);

    config
        .command
        .arg("test task from test 'simple_create_task'")
        .assert()
        .stdout(predicate::str::contains("Task added").count(1))
        .success();
}

#[test]
#[ignore = "cvcap bin run (slow)"]
fn status_reports_logged_in_and_configured_default_list() {
    let mut config = TestConfig::new(true, true);

    config
        .command
        .arg("status")
        .assert()
        .stdout(predicate::str::contains(TEST_LIST_NAME).count(1))
        .stdout(predicate::str::contains("✅").count(1))
        .success();
}

#[test]
#[ignore = "cvcap bin run (slow)"]
fn status_reports_default_list_not_configured() {
    let mut config = TestConfig::new(true, false);

    config
        .command
        .arg("status")
        .assert()
        .stdout(predicate::str::contains("❌").count(1))
        .success();
}

#[test]
#[ignore = "cvcap bin run (slow)"]
fn status_reports_user_not_logged_in() {
    let mut config = TestConfig::new(false, true);

    config
        .command
        .arg("status")
        .assert()
        .stdout(predicate::str::contains("❌").count(1))
        .success();
}

#[test]
#[ignore = "cvcap bin run (slow)"]
fn cannot_combine_add_main_command_with_options() {
    args_should_conflict(vec!["a task", "-l", "-v"]);
}


// https://github.com/crispinb/cvcap/issues/14
#[test]
#[ignore = "cvcap bin run (slow)"]
fn shows_must_login_message_when_token_refresh_fails() {
    let mut config = TestConfig::new(true, true);
    // invalidate the token so a refresh will fail
    test_creds::save_api_token_to_keyring("invalid token", &config.keyring_service_name);

    config
        .command
        .arg("task to add")
        .assert()
        .stderr(predicate::str::contains(
            "You have been logged out of the Checkvist API",
        ))
        .failure();
}

#[test]
#[ignore = "cvcap bin run (slow)"]
fn add_task_from_stdin() {
    let mut config = TestConfig::new(true, true);

    config
        .command
        .arg("add")
        .arg("-s")
        .pipe_stdin("tests/data/task.md")
        .unwrap()
        .assert()
        .stdout(predicate::str::contains("This is a test task"))
        .success();
}

#[test]
#[ignore = "cvcap bin run (slow)"]
fn s_and_c_flags_conflict() {
    args_should_conflict(vec!["add", "-c", "-s"]);
}

#[test]
#[ignore = "cvcap bin run (slow)"]
fn c_flag_conflicts_with_content_arg() {
    args_should_conflict(vec!["add", "content", "-c"]);
}

#[test]
#[ignore = "cvcap bin run (slow)"]
fn s_flag_conflicts_with_content_arg() {
    args_should_conflict(vec!["add", "content", "-s"]);
}


// This attempt to test for -s with no pipe doesn't work.
//`cvcap add -s` has a 0 result code here,
// and adds an empty task to the test list. 
// It fails (as it should) run from a shell.
// Something to do with atty/assert_cmd interaction?
// #[test]
// #[ignore = "cvcap bin run (slow)"]
// fn s_option_without_pipe_errors() {
//     let mut config = TestConfig::new(true, true);

//     config
//         .command
//         .arg("add")
//         .arg("-s")
//         .assert()
//         // .stderr(predicate::str::contains("SOME ERROR"))
//         .failure();
// }

struct TestConfig {
    logged_in: bool,
    command: assert_cmd::Command,
    keyring_service_name: String,
    // TempDir gets deleted when dropped, so we hold for test duration
    _temp_dir: TempDir,
}

impl std::ops::Drop for TestConfig {
    fn drop(&mut self) {
        if self.logged_in {
            test_creds::delete_api_token(&self.keyring_service_name);
        }
    }
}

impl TestConfig {
    fn new(logged_in: bool, configured: bool) -> Self {
        unsafe {
            CREATE_TEST_LIST.call_once(|| {
                TEST_LIST = get_or_create_test_list();
            });
        }

        let keyring_service_name = if logged_in {
            let service_name = random_service_name();
            let api_token = std::env::var_os(API_KEY_ENV).unwrap();
            test_creds::save_api_token_to_keyring(&api_token.to_string_lossy(), &service_name);
            service_name
        } else {
            "cvcap-cli_integration_tests-nonexistent-keyring-service-name".into()
        };
        let temp_dir = TempDir::new().unwrap();
        let config_file_path = if configured {
            unsafe {
                let cvcap_config = TestCvcapRunConfig {
                    list_id: TEST_LIST.id,
                    list_name: TEST_LIST.name.clone(),
                };
                create_config_file(cvcap_config, &temp_dir)
            }
        } else {
            "cvcap-cli_integration_tests-nonexistent-config-file_path".into()
        };

        let mut cmd = assert_cmd::Command::cargo_bin("cvcap").unwrap();
        cmd.env("CVCAP_CREDENTIAL_ID", &keyring_service_name)
            .env("CVCAP_CONFIG_FILE_PATH", &config_file_path);

        TestConfig {
            logged_in,
            command: cmd,
            keyring_service_name,
            _temp_dir: temp_dir,
        }
    }
}

fn args_should_conflict(args: Vec<&str>) {
    let mut config = TestConfig::new(true, true);

    config
        .command
        .args(args)
        .assert()
        .failure();
}

fn random_service_name() -> String {
    format!("cvcap-test-{}", Uuid::new_v4())
}

fn create_config_file(config: TestCvcapRunConfig, temp_dir: &TempDir) -> path::PathBuf {
    let random_file_name = temp_dir.child(Uuid::new_v4().to_string());
    config.write_to_new_file(&random_file_name).unwrap();

    random_file_name
}

fn get_or_create_test_list() -> Checklist {
    let api_token = std::env::var_os(API_KEY_ENV).expect("CVCAP_API_TOKEN must be set");
    let client = CheckvistClient::new(
        "https://checkvist.com/".into(),
        api_token.to_string_lossy().to_string(),
        |_| (),
    );

    let lists = client.get_lists().unwrap();
    if let Some(list) = lists.into_iter().find(|list| list.name == TEST_LIST_NAME) {
        list
    } else {
        client.add_list(TEST_LIST_NAME).unwrap()
    }
}

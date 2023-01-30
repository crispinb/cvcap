use core::fmt;
use log::{error, info};
use std::cell::RefCell;
use std::collections::HashMap;
use std::vec;

use serde::{Deserialize, Serialize};
use url::Url;

#[derive(PartialEq, Eq, Debug, Deserialize, Serialize)]
pub struct Checklist {
    pub id: u32,
    pub name: String,
    // TODO: convert to a date type of some sort when needed
    pub updated_at: String,
    pub task_count: u16,
}

/// Generic location of an item in a Checkvist list.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct CheckvistLocation {
    pub list_id: u32,
    pub parent_task_id: Option<u32>,
}

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, Clone)]
pub struct Task {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<u32>,
    pub content: String,
    pub position: u16,
    pub parent_id: Option<u32>,
}

#[derive(Deserialize)]
struct ApiToken {
    token: String,
}

type Result<T> = std::result::Result<T, CheckvistError>;

// TODO: check all the variant sizes - clippy complains this is too large
//https://rust-lang.github.io/rust-clippy/master/index.html#result_large_err
// clippy ` cargo clippy --workspace -- -A "clippy::result_large_err"` for now
#[derive(Debug)]
pub enum CheckvistError {
    InvalidParentIdError,
    InvalidListError,
    InvalidTaskError,
    UnknownError { message: String },
    NetworkError(ureq::Error),
    // used by serde_json for decoding errors
    IoError(std::io::Error),
    TokenRefreshFailedError,
}

impl fmt::Display for CheckvistError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::IoError(ref err) => write!(f, "{:?}", err),
            Self::NetworkError(ref err) => write!(f, "{:?}", err),
            Self::InvalidListError => write!(f, "You tried to add a task to a list that can't be found, or you don't have permission to access"),
            Self::InvalidTaskError => write!(f, "You tried to add a task to a parent task that can't be found, or you don't have permission to access"),
            Self::InvalidParentIdError => write!(f, "You  tried to add a task to a parent task that can't be found"),
            Self::UnknownError { ref message } => write!(f, "{}", message),
            Self::TokenRefreshFailedError => write!(f, "Could not refresh token"),
        }
    }
}

impl std::error::Error for CheckvistError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match *self {
            Self::IoError(ref err) => Some(err),
            Self::NetworkError(ref err) => Some(err),
            Self::TokenRefreshFailedError => None,
            _ => None,
        }
    }
}

impl From<ureq::Error> for CheckvistError {
    fn from(err: ureq::Error) -> Self {
        match err {
            ureq::Error::Status(status, response) => {
                let Ok(response_json) = response.into_json::<HashMap<String, String>>() else {
                    return CheckvistError::UnknownError {
                       message: "Couldn't parse ureq error text as json".into(),
                    };
                };
                let default_msg = String::new();
                let message = response_json.get("message").unwrap_or(&default_msg);
                if status == 403
                    && message.contains("The list doesn't exist or is not available to you")
                {
                    CheckvistError::InvalidListError
                } else if status == 400 && message.contains("Invalid parent_id") {
                    CheckvistError::InvalidParentIdError
                } else {
                    // would prefer to include the ureq::Error in a NetworkError, but into_json
                    // consumes it
                    CheckvistError::UnknownError {
                        message: format!(
                            "Unexpected network error received from ureq. Status: {}",
                            status
                        ),
                    }
                }
            }
            // ureq::Errror::Transport
            _ => CheckvistError::NetworkError(err),
        }
    }
}

impl From<std::io::Error> for CheckvistError {
    fn from(err: std::io::Error) -> Self {
        CheckvistError::IoError(err)
    }
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
enum ApiResponse<T> {
    OkCheckvistItem(T),
    OkCheckvistList(Vec<T>),
    CheckvistApiError { message: String },
}

pub struct CheckvistClient {
    base_url: Url,
    api_token: RefCell<String>,
    // should we need multiple callbacks, replace this with a vec of trait objects
    token_refresh_callback: Box<dyn Fn(&str)>,
}

impl fmt::Debug for CheckvistClient {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CheckvistClient")
            .field("base_url", &self.base_url)
            .field("api_token", &self.api_token)
            .finish_non_exhaustive()
    }
}

impl CheckvistClient {
    pub fn new(base_url: &str, api_token: &str, on_token_refresh: Box<dyn Fn(&str)>) -> Self {
        Self {
            base_url: Url::parse(base_url).expect("Bad base url supplied"),
            api_token: RefCell::new(api_token.into()),
            token_refresh_callback: on_token_refresh,
        }
    }

    pub fn get_token(base_url: &str, username: &str, remote_key: &str) -> Result<String> {
        let url = CheckvistClient::build_endpoint(
            &Url::parse(base_url).expect("Bad base URL supplied"),
            vec!["/auth/login.json?version=2"],
        );

        let response: ApiToken = ureq::post(url.as_str())
            .send_json(ureq::json!({"username": username, "remote_key": remote_key}))?
            .into_json()?;

        Ok(response.token)
    }

    pub fn refresh_token(&self) -> Result<()> {
        let url = CheckvistClient::build_endpoint(
            &self.base_url,
            vec!["/auth/refresh_token.json?version=2"],
        );

        info!("Refreshing api token");
        let response: ApiToken = ureq::post(url.as_str())
            .send_json(ureq::json!({"old_token": self.api_token.borrow().clone()}))
            // *any* error here means the token refresh failed
            .map_err(|_| CheckvistError::TokenRefreshFailedError)?
            .into_json()?;

        *self.api_token.borrow_mut() = response.token.clone();
        info!("Refreshed api token");
        (self.token_refresh_callback)(&response.token);

        Ok(())
    }

    /// Checks whether or not the location exists
    /// Returns Ok(true) if so, Ok(false) if not
    /// Any Err value indicates something unexpected (network, auth,etc)
    pub fn is_location_valid(&self, location: &CheckvistLocation) -> Result<bool> {
        match location.parent_task_id {
            None => match self.get_list(location.list_id) {
                Ok(_) => Ok(true),
                Err(e) => {
                    if let CheckvistError::InvalidListError = e {
                        Ok(false)
                    } else {
                        Err(e)
                    }
                }
            },
            Some(parent_task_id) => match self.get_task(location.list_id, parent_task_id) {
                Ok(_) => Ok(true),
                Err(e) => {
                    if let CheckvistError::InvalidTaskError = e {
                        Ok(false)
                    } else {
                        Err(e)
                    }
                }
            },
        }
    }

    pub fn get_lists(&self) -> Result<Vec<Checklist>> {
        let url = CheckvistClient::build_endpoint(&self.base_url, vec!["/checklists.json"]);

        let response = self.checkvist_get(url)?.into_json()?;

        self.to_results(response)
    }

    pub fn get_list(&self, list_id: u32) -> Result<Checklist> {
        let url = CheckvistClient::build_endpoint(
            &self.base_url,
            vec!["/checklists/", &list_id.to_string(), ".json"],
        );

        let response = self.checkvist_get(url)?.into_json()?;

        self.to_result(response)
    }

    pub fn add_list(&self, list_name: &str) -> Result<Checklist> {
        let url = CheckvistClient::build_endpoint(&self.base_url, vec!["/checklists", ".json"]);

        let response = self
            .checkvist_post(url, HashMap::from([("name", list_name)]))?
            .into_json()?;

        self.to_result(response)
    }

    /// Checkvist returns the task with its parents (if any)
    pub fn get_task(&self, list_id: u32, task_id: u32) -> Result<Vec<Task>> {
        let url = CheckvistClient::build_endpoint(
            &self.base_url,
            vec![
                "/checklists/",
                &list_id.to_string(),
                "/tasks/",
                &task_id.to_string(),
                ".json",
            ],
        );

        let response = self.checkvist_get(url)?.into_json()?;

        self.to_results(response)
    }

    pub fn get_tasks(&self, list_id: u32) -> Result<Vec<Task>> {
        let url = CheckvistClient::build_endpoint(
            &self.base_url,
            vec!["/checklists/", &list_id.to_string(), "/tasks.json"],
        );

        let response = self.checkvist_get(url)?.into_json()?;

        self.to_results(response)
    }

    pub fn add_task(&self, list_id: u32, task: &Task) -> Result<Task> {
        let url = CheckvistClient::build_endpoint(
            &self.base_url,
            vec!["/checklists/", &list_id.to_string(), "/tasks.json"],
        );

        let response = self.checkvist_post(url, task)?.into_json()?;

        error!("response: {:?}", response);
        self.to_result(response)
    }

    // TODO: - REFACTOR: combine get & post methods
    fn checkvist_post<T: serde::Serialize>(&self, url: Url, payload: T) -> Result<ureq::Response> {
        let request =
            ureq::post(url.as_str()).set("X-Client-token", &self.api_token.borrow().clone());
        let response = request.send_json(&payload).or_else(|err| {
            match err {
                ureq::Error::Status(401, _) => {
                    match self.refresh_token() {
                        // we have a new token. Try the request again
                        Ok(_) => {
                            // Self has a new token, so we must rebuild the request
                            let request = ureq::post(url.as_str())
                                .set("X-Client-token", &self.api_token.borrow().clone());
                            Ok(request.send_json(&payload)?)
                        }

                        // CheckvistError::TokenRefreshFailedError
                        Err(err) => Err(err),
                    }
                }
                // let CheckvistError From handle
                err => Err(err)?,
            }
        })?;

        Ok(response)
    }

    // TODO: - REFACTOR: combine get & post methods
    fn checkvist_get(&self, url: Url) -> Result<ureq::Response> {
        let request =
            ureq::get(url.as_str()).set("X-Client-token", &self.api_token.borrow().clone());
        let response = request.call().or_else(|err| {
            match err {
                ureq::Error::Status(401, _) => {
                    match self.refresh_token() {
                        // we have a new token. Try the request again
                        Ok(_) => {
                            // Self has a new token, so we must rebuild the request
                            let request = ureq::get(url.as_str())
                                .set("X-Client-token", &self.api_token.borrow().clone());
                            Ok(request.call()?)
                        }

                        // CheckvistError::TokenRefreshFailedError
                        Err(err) => Err(err),
                    }
                }

                err => Err(err)?,
            }
        })?;

        Ok(response)
    }

    // Utility Methods

    // TODO:  RESEARCH NEEDED:
    //        how to merge with to_result?
    // check JSON implementation in Programming Rust, p.234 (Enums ch).
    // For arrays it nests vecs of itself (aot APIResponse which has Vec<T>)
    fn to_results<T>(&self, response: ApiResponse<T>) -> Result<Vec<T>> {
        match response {
            ApiResponse::OkCheckvistList(v) => Ok(v),
            ApiResponse::CheckvistApiError { message } => {
                Err(CheckvistError::UnknownError { message })
            }
            _ => Err(CheckvistError::UnknownError {
                message: String::new(),
            }),
        }
    }

    fn to_result<T>(&self, response: ApiResponse<T>) -> Result<T> {
        match response {
            ApiResponse::OkCheckvistItem(returned_struct) => Ok(returned_struct),
            // as I don't know how to merge the 2 to_results, and we must deal with all responses here:
            ApiResponse::OkCheckvistList(_t) => {
                error!("Checkvist API returned JSON decoded to unexpected type");
                panic!("Something irrecoverable happened")
            }
            ApiResponse::CheckvistApiError { message } => {
                Err(CheckvistError::UnknownError { message })
            }
        }
    }

    // Utility Functions
    // TODO:  RESEARCH NEEDED:
    //        wanted to replace Vec<&str> with Vec<std::path::Path>, but get type error
    fn build_endpoint(base_url: &Url, segments: Vec<&str>) -> Url {
        base_url
            .join(&segments.concat())
            .expect("Error building endpoing (shouldn't happen as base_url is known good")
    }
}

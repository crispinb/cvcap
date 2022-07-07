use core::fmt;
use log::{error, info};
use std::cell::RefCell;
use std::vec;

use serde::{Deserialize, Serialize};
use url::Url;

#[derive(PartialEq, Debug, Deserialize, Serialize)]
// only need PartialEq for test, but this doesn't work
// because: integration tests build differently?
// #[cfg_attr(all(test), derive(PartialEq))]
pub struct Checklist {
    pub id: u32,
    pub name: String,
    // TODO: automatically convert to a date type of some sort when needed
    pub updated_at: String,
    pub task_count: u16,
}

#[derive(PartialEq, Debug, Serialize, Deserialize, Clone)]
pub struct Task {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<u32>,
    pub content: String,
    pub position: u16,
}

// struct seems a bit overwrought for this, but it turns out simpler
// than messing with serde_json::Value (see https://play.rust-lang.org/?gist=9e64149fe110c686619185a783e78fcc&version=nightly)
#[derive(Deserialize)]
struct ApiToken {
    token: String,
}

#[derive(Debug)]
pub enum CheckvistError {
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
            Self::UnknownError { message: _ } => None,
            Self::TokenRefreshFailedError => None,
        }
    }
}

impl From<ureq::Error> for CheckvistError {
    fn from(err: ureq::Error) -> Self {
        CheckvistError::NetworkError(err)
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

// TODO - RESEARCH NEEDED:
//        Ownership problems trying to implement this on ApiResponse
//        (that don't occur when implementing on CheckVistClilent ??)
// impl<T> ApiResponse<T> {
//     fn to_results(&self) -> Result<Vec<T>, CheckvistError> {
//         match *self {
//             ApiResponse::ValidCheckvistList(ref v) => Ok(v),
//             ApiResponse::CheckvistApiError { ref message } => Err(CheckvistError::UnknownError {
//                 message: message.to_string(),
//             }),
//             _ => Err(CheckvistError::UnknownError {
//                 message: String::new(),
//             }),
//         }
//     }
// }

pub struct CheckvistClient {
    base_url: Url,
    api_token: RefCell<String>,
    token_refresh_callback: fn(&str) -> (), 
}

impl CheckvistClient {
    pub fn new(base_url: String, api_token: String, on_token_refresh: fn(&str) -> ()) -> Self {
        Self {
            base_url: Url::parse(&base_url).expect("Bad base url supplied"),
            api_token: RefCell::new(api_token),
            token_refresh_callback: on_token_refresh,
        }
    }

    // TODO: perhaps build into CheckvistClient::new ? Then we'd need 2 news (one with token)
    pub fn get_token(
        base_url: String,
        username: String,
        remote_key: String,
    ) -> Result<String, CheckvistError> {
        let url = CheckvistClient::build_endpoint(
            &Url::parse(&base_url).expect("Bad base URL supplied"),
            vec!["/auth/login.json?version=2"],
        );

        let response: ApiToken = ureq::post(url.as_str())
            .send_json(ureq::json!({"username": username, "remote_key": remote_key}))?
            .into_json()?;

        Ok(response.token)
    }

    pub fn refresh_token(&self) -> Result<(), CheckvistError> {
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

    pub fn get_lists(&self) -> Result<Vec<Checklist>, CheckvistError> {
        let url = CheckvistClient::build_endpoint(&self.base_url, vec!["/checklists.json"]);

        let response = self.checkvist_get(url)?.into_json()?;

        self.to_results(response)
    }

    pub fn get_list(&self, list_id: u32) -> Result<Checklist, CheckvistError> {
        let url = CheckvistClient::build_endpoint(
            &self.base_url,
            vec!["/checklists/", &list_id.to_string(), ".json"],
        );

        let response = self.checkvist_get(url)?.into_json()?;

        self.to_result(response)
    }

    pub fn get_tasks(&self, list_id: u32) -> Result<Vec<Task>, CheckvistError> {
        let url = CheckvistClient::build_endpoint(
            &self.base_url,
            vec!["/checklists/", &list_id.to_string(), "/tasks.json"],
        );

        let response = self.checkvist_get(url)?.into_json()?;

        self.to_results(response)
    }

    pub fn add_task(&self, list_id: u32, task: &Task) -> Result<Task, CheckvistError> {
        let url = CheckvistClient::build_endpoint(
            &self.base_url,
            vec!["/checklists/", &list_id.to_string(), "/tasks.json"],
        );

        let response = self.checkvist_post(url, task)?.into_json()?;

        self.to_result(response)
    }

    // TODO - REFACTOR: combine get & post methods
    fn checkvist_post<T: serde::Serialize>(
        &self,
        url: Url,
        payload: T,
    ) -> Result<ureq::Response, CheckvistError> {
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
                            request
                                .send_json(&payload)
                                // without this, the match (which is the return value of the or_else
                                // closure) is of type Result<Response, ureq::Error>.
                                // That's OK in this arm with Ok(Response), but conflicts
                                // with the Err arm which returns an Err(CheckvistError)
                                .map_err(CheckvistError::NetworkError)
                        }

                        // failed to refresh token
                        Err(err) => Err(err),
                    }
                }
                err => Err(CheckvistError::NetworkError(err)),
            }
        })?;

        Ok(response)
    }

    fn checkvist_get(&self, url: Url) -> Result<ureq::Response, CheckvistError> {
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
                            request
                                .call()
                                // without this, the match (which is the return value of the or_else
                                // closure) is of type Result<Response, ureq::Error>.
                                // That's OK in this arm with Ok(Response), but conflicts
                                // with the Err arm which returns an Err(CheckvistError)
                                .map_err(CheckvistError::NetworkError)
                        }

                        // failed to refresh token
                        Err(err) => Err(err),
                    }
                }
                err => Err(CheckvistError::NetworkError(err)),
            }
        })?;

        Ok(response)
    }

    // Utility Methods

    // TODO - RESEARCH NEEDED:
    //        how to merge with to_result?
    // check JSON implementation in Programming Rust, p.234 (Enums ch).
    // For arrays it nests vecs of itself (aot APIResponse which has Vec<T>)
    fn to_results<T>(&self, response: ApiResponse<T>) -> Result<Vec<T>, CheckvistError> {
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

    fn to_result<T>(&self, response: ApiResponse<T>) -> Result<T, CheckvistError> {
        match response {
            ApiResponse::OkCheckvistItem(returned_struct) => Ok(returned_struct),
            // as I don't know how to merge the 2 to_results, and we must deal with all responses here:
            ApiResponse::OkCheckvistList(_t) => {
                error!("Checkvist API returned JSON decoded to unexpected type");
                panic!("Something irrecoverable happened")
            }
            // Q: should we parse out known errors here? (eg auth). But it's all based on an (only assumed stable) 'message' string so would hardly be reliable, but then could have a fallback type
            ApiResponse::CheckvistApiError { message } => {
                Err(CheckvistError::UnknownError { message })
            }
        }
    }

    // Utility Functions

    // TODO - RESEARCH NEEDED:
    //        wanted to replace Vec<&str> with Vec<std::path::Path>, but get type error
    fn build_endpoint(base_url: &Url, segments: Vec<&str>) -> Url {
        base_url
            .join(&segments.concat())
            .expect("Error building endpoing (shouldn't happen as base_url is known good")
    }
}

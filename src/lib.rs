mod checkvist_types;
pub mod sqlite_store;
pub mod sqlite_client;

pub use checkvist_types::{CheckvistClient, Checklist, Task, CheckvistError, CHECKVIST_DATE_FORMAT};

use log::{error, info};
use std::cell::RefCell;
use std::collections::HashMap;
use std::vec;

use serde::Deserialize;
use url::Url;

// struct seems a bit overwrought for this, but it turns out simpler
// than messing with serde_json::Value (see https://play.rust-lang.org/?gist=9e64149fe110c686619185a783e78fcc&version=nightly)
#[derive(Deserialize)]
struct ApiToken {
    token: String,
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


pub struct ApiClient {
    base_url: Url,
    api_token: RefCell<String>,
    token_refresh_callback: fn(&str) -> (),
}

impl ApiClient {
    pub fn new(base_url: String, api_token: String, on_token_refresh: fn(&str) -> ()) -> Self {
        Self {
            base_url: Url::parse(&base_url).expect("Bad base url supplied"),
            api_token: RefCell::new(api_token),
            token_refresh_callback: on_token_refresh,
        }
    }

    pub fn get_token(
        base_url: String,
        username: String,
        remote_key: String,
    ) -> Result<String, CheckvistError> {
        let url = ApiClient::build_endpoint(
            &Url::parse(&base_url).expect("Bad base URL supplied"),
            vec!["/auth/login.json?version=2"],
        );

        let response: ApiToken = ureq::post(url.as_str())
            .send_json(ureq::json!({"username": username, "remote_key": remote_key}))?
            .into_json()?;

        Ok(response.token)
    }

    pub fn refresh_token(&self) -> Result<(), CheckvistError> {
        let url = ApiClient::build_endpoint(
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
            .expect("Error building endpoint (shouldn't happen as base_url is known good")
    }
}

impl CheckvistClient for ApiClient {

    fn get_lists(&self) -> Result<Vec<Checklist>, CheckvistError> {
        let url = ApiClient::build_endpoint(&self.base_url, vec!["/checklists.json"]);

        let response = self.checkvist_get(url)?.into_json()?;

        self.to_results(response)
    }

    fn get_list(&self, list_id: u32) -> Result<Checklist, CheckvistError> {
        let url = ApiClient::build_endpoint(
            &self.base_url,
            vec!["/checklists/", &list_id.to_string(), ".json"],
        );

        let response = self.checkvist_get(url)?.into_json()?;

        self.to_result(response)
    }

    fn add_list(&self, list_name: &str) -> Result<Checklist, CheckvistError> {
        let url = ApiClient::build_endpoint(&self.base_url, vec!["/checklists", ".json"]);

        let response = self
            .checkvist_post(url, HashMap::from([("name", list_name)]))?
            .into_json()?;

        self.to_result(response)
    }

    fn get_tasks(&self, list_id: u32) -> Result<Vec<Task>, CheckvistError> {
        let url = ApiClient::build_endpoint(
            &self.base_url,
            vec!["/checklists/", &list_id.to_string(), "/tasks.json"],
        );

        let response = self.checkvist_get(url)?.into_json()?;

        self.to_results(response)
    }

    fn add_task(&self, list_id: u32, task: &Task) -> Result<Task, CheckvistError> {
        let url = ApiClient::build_endpoint(
            &self.base_url,
            vec!["/checklists/", &list_id.to_string(), "/tasks.json"],
        );

        let response = self.checkvist_post(url, task)?.into_json()?;

        self.to_result(response)
    }

}

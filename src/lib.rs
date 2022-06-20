use core::fmt;
use std::vec;

use serde::{Deserialize, Serialize};
use url::Url;

// TODO: If we don't need PartialEq other than for tests, conditionally compile attribute for tests only https://doc.rust-lang.org/reference/conditional-compilation.html.
#[derive(PartialEq, Debug, Deserialize, Serialize)]
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

#[derive(Debug)]
pub enum CheckvistError {
    UnknownError { message: String },
    NetworkError(ureq::Error),
    // used by serde_json for decoding errors
    IoError(std::io::Error),
}

impl fmt::Display for CheckvistError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::IoError(ref err) => write!(f, "{:?}", err),
            Self::NetworkError(ref err) => write!(f, "{:?}", err),
            Self::UnknownError { ref message } => write!(f, "{}", message),
        }
    }
}

impl std::error::Error for CheckvistError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match *self {
            Self::IoError(ref err) => Some(err),
            Self::NetworkError(ref err) => Some(err),
            Self::UnknownError { message: _ } => None,
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

#[derive(Debug)]
pub struct CheckvistClient {
    base_url: Url,
    api_token: String,
}

impl CheckvistClient {
    pub fn new(base_url: String, api_token: String) -> Self {
        Self {
            base_url: Url::parse(&base_url).expect("Bad base url supplied"),
            api_token,
        }
    }

    pub fn get_lists(&self) -> Result<Vec<Checklist>, CheckvistError> {
        let url = self.build_endpoint(vec!["/checklists.json"]);

        let response: ApiResponse<Checklist> = ureq::get(url.as_str())
            .set("X-Client-Token", &self.api_token)
            .call()?
            .into_json()?;

        self.to_results(response)
    }

    pub fn get_list(&self, list_id: u32) -> Result<Checklist, CheckvistError> {
        let url = self.build_endpoint(vec!["/checklists/", &list_id.to_string(), ".json"]);

        let response: ApiResponse<Checklist> = ureq::get(url.as_str())
            .set("X-Client-token", &self.api_token)
            .call()?
            .into_json()?;

        self.to_result(response)
    }

    pub fn get_tasks(&self, list_id: u32) -> Result<Vec<Task>, CheckvistError> {
        let url = self.build_endpoint(vec!["/checklists/", &list_id.to_string(), "/tasks.json"]);

        let response: ApiResponse<Task> = ureq::get(url.as_str())
            .set("X-Client-token", &self.api_token)
            .call()?
            .into_json()?;

        self.to_results(response)
    }

    pub fn add_task(&self, list_id: u32, task: Task) -> Result<Task, CheckvistError> {
        let url = self.build_endpoint(vec!["/checklists/", &list_id.to_string(), "/tasks.json"]);

        let response: ApiResponse<Task> = ureq::post(url.as_str())
            .set("X-Client-Token", &self.api_token)
            .send_json(task)?
            .into_json()?;
        self.to_result(response)
    }

    // TODO - RESEARCH NEEDED:
    //        how to merge with to_result?
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
            ApiResponse::OkCheckvistList(_v) => panic!("Should never get here"),
            // Q: should we parse out known errors here? (eg auth). But it's all based on an (only assumed stable) 'message' string so would hardly be reliable, but then could have a fallback type
            ApiResponse::CheckvistApiError { message } => {
                Err(CheckvistError::UnknownError { message })
            }
        }
    }

    // TODO - RESEARCH NEEDED: 
    //        wanted to replace Vec<&str> with Vec<std::path::Path>, but get type error
    fn build_endpoint(&self, segments: Vec<&str>) -> Url {
        self.base_url
            .join(&segments.concat())
            .expect("Error building endpoing (shouldn't happen as base_url is known good")
    }
}

#![allow(dead_code)]
use core::fmt;
use std::vec;

use serde::{Deserialize, Serialize};
use url::Url;

// curl --header "X-Client-Token: [token]" "https://checkvist.com/checklists.json"
//     .get("https://checkvist.com/checklists.json")
// current token: HRpvPJqF4uvwVR8jQ3mkiqlwCm7Y6n
// if token is bad or expired, receive: `{"message":"Unauthenticated: no valid authentication data in request"}`
// json return:
//Object({"archived": Bool(false), "created_at": String("2020/09/13 21:45:52 +0000"), "id": Number(774394), "item_count": Number(16), "markdown?": Bool(true), "name": String("devtest"), "options": Number(3), "percent_completed": Number(0.0), "public": Bool(false), "read_only": Bool(false), "related_task_ids": Null, "tags": Object({"to_review": Bool(false)}), "tags_as_text": String("to_review"), "task_completed": Number(0), "task_count": Number(16), "updated_at": String("2022/04/26 18:41:15 +1000"), "user_count": Number(1), "user_updated_at": String("2022/04/26 18:41:15 +1000")})

// If we don't need PartialEq other than for tests, we can conditionally compile attribute for tests only https://doc.rust-lang.org/reference/conditional-compilation.html.
#[derive(PartialEq, Debug, Deserialize, Serialize)]
pub struct Checklist {
    pub id: i32,
    pub name: String,
    // TODO: automatically convert to a date type of some sort
    pub updated_at: String,
    pub task_count: u16,
}

#[derive(PartialEq, Debug, Serialize, Deserialize, Clone)]
pub struct Task {
    pub content: String,
    pub id: i32,
    pub position: i16
}

#[derive(Debug)]
pub enum CheckvistError {
    UnknownError { message: String },
    NetworkError(ureq::Error),
    // used by serde_json for decoding errors
    IoError(std::io::Error),
}

// TODO - REFACTOR: format error messages appropriately (& test)
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

// TODO: decide if this is needed (if so would hold a ref to a ureq Agent)
#[derive(Debug)]
struct HttpClient {}
impl HttpClient {
    pub fn new() -> HttpClient {
        HttpClient {}
    }
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
enum ApiResponse<T> {
    ValidCheckvistType(T),
    ValidCheckvistList(Vec<T>),
    CheckvistApiError { message: String },
}

// TODO - RESEARCH UNSOLVED PROBLEM:
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
    client: HttpClient,
    base_url: Url,
    api_token: String,
}

impl CheckvistClient {
    pub fn new(base_url: String, api_token: String) -> Self {
        Self {
            client: HttpClient::new(),
            base_url: Url::parse(&base_url).expect("Bad base url supplied"),
            api_token,
        }
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

    // TODO - RESEARCH UNSOLVED PROBLEM:
    //        how to merge with to_result?
    fn to_results<T>(&self, response: ApiResponse<T>) -> Result<Vec<T>, CheckvistError> {
        match response {
            ApiResponse::ValidCheckvistList(v) => Ok(v),
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
            ApiResponse::ValidCheckvistType(returned_struct) => Ok(returned_struct),
            // as I don't know how to merge the 2 to_results, and we must deal with all responses here:
            ApiResponse::ValidCheckvistList(_v) => panic!("Should never get here"),
            // Q: should we parse out known errors here? (eg auth). But it's all based on an (only assumed stable) 'message' string so would hardly be reliable, but then could have a fallback type
            ApiResponse::CheckvistApiError { message } => {
                Err(CheckvistError::UnknownError { message })
            }
        }
    }

    fn build_endpoint(&self, segments: Vec<&str>) -> Url {
        self.base_url
            .join(&segments.concat())
            .expect("Error building endpoing (shouldn't happen as base_url is known good")
    }
}

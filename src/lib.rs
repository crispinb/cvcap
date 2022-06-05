#![allow(dead_code)]
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

#[derive(PartialEq, Debug, Serialize, Deserialize)]
pub struct Task {
    pub content: String,
    pub id: i32,
}

// REFACTOR: now we've eliminated PartialEq on errors, use Box<dyn Err> to store inner error 
#[derive(Debug, Clone)]
pub enum CheckvistError {
    UnknownError { message: String },
    // for now I don't know how to merge get_results and get_result
    DumbCBApiError,
    NetworkError { status: u16, message: String },
}

impl From<ureq::Error> for CheckvistError {
    fn from(original: ureq::Error) -> Self {
        match original {
            ureq::Error::Status(code, _) => Self::NetworkError {
                status: code,
                message: "".into(),
            },
            ureq::Error::Transport(t) => Self::NetworkError {
                status: 0,
                message: t.kind().to_string(),
            },
        }
    }
}

impl From<std::io::Error> for CheckvistError {
    fn from(original: std::io::Error) -> Self {
        println!("inner errlr: {:?}", original);
        Self::UnknownError {
            message: original.kind().to_string(),
        }
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

// thus far failing attempts to implement to_results on ApiResponse
// impl <L> ApiResponse<L> {
//     fn to_results<T>(&self) -> Result<Vec<T>, CheckvistError> {
//         match self  {
//             ApiResponse::ValidCheckvistList(v) => Ok(v),
//             ApiResponse::CheckvistApiError { message } => Err(CheckvistError::UnknownError { message: message.to_string() }),
//             _ => Err( CheckvistError::UnknownError { message: String::new() } ),
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
        let list_id_segment = list_id.to_string();
        let url = self.build_endpoint(vec!["/checklists/", &list_id_segment, ".json"]);

        let response: ApiResponse<Checklist> = ureq::get(url.as_str())
            .set("X-Client-token", &self.api_token)
            .call()?
            .into_json()?;

        // TODO: or make this a method on ApiResponse?
        self.to_result(response)
    }

    pub fn get_tasks(&self, list_id: u32) -> Result<Vec<Task>, CheckvistError> {
        let list_id_segment = list_id.to_string();
        let url = self.build_endpoint(vec!["/checklists/", &list_id_segment, "/tasks.json"]);

        // TODO: possibly extract ureq Agent (read up on purpose)
        let response: ApiResponse<Task> = ureq::get(url.as_str())
            .set("X-Client-token", &self.api_token)
            .call()?
            .into_json()?;

        self.to_results(response)
    }

    // TODO: would prefer to implement these on ApiResponse, but so far failed (type problems)
    // TODO: can this be merged with to_result?
    fn to_results<T>(&self, response: ApiResponse<T>) -> Result<Vec<T>, CheckvistError> {
        match response  {
            ApiResponse::ValidCheckvistList(v) => Ok(v),
            ApiResponse::CheckvistApiError { message } => Err(CheckvistError::UnknownError { message }),
            _ => Err( CheckvistError::UnknownError { message: String::new() } ),
        }

    }

    fn to_result<T>(&self, response: ApiResponse<T>) -> Result<T, CheckvistError> {
        match response {
            ApiResponse::ValidCheckvistType(returned_struct) => Ok(returned_struct),
            ApiResponse::ValidCheckvistList(_v) => Err( CheckvistError::DumbCBApiError ),
            // Q: should we parse out known errors here? (eg auth). But it's all based on an (only assumed stable) 'message' string so would hardly be reliable, but then could have a fallback type
            ApiResponse::CheckvistApiError { message } => Err(CheckvistError::UnknownError { message }),
        }
    }

    fn build_endpoint(&self, segments: Vec<&str>) -> Url {
        self.base_url
            .join(&segments.concat())
            .expect("Error building endpoing (shouldn't happen as base_url is known good")
    }
}

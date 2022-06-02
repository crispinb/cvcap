#![allow(dead_code)]
use serde::{Deserialize, Serialize};
use url::Url;

// curl --header "X-Client-Token: [token]" "https://checkvist.com/checklists.json"
//     .get("https://checkvist.com/checklists.json")
// current token: HRpvPJqF4uvwVR8jQ3mkiqlwCm7Y6n
// if token is bad or expired, receive: `{"message":"Unauthenticated: no valid authentication data in request"}`
// json return:
//Object({"archived": Bool(false), "created_at": String("2020/09/13 21:45:52 +0000"), "id": Number(774394), "item_count": Number(16), "markdown?": Bool(true), "name": String("devtest"), "options": Number(3), "percent_completed": Number(0.0), "public": Bool(false), "read_only": Bool(false), "related_task_ids": Null, "tags": Object({"to_review": Bool(false)}), "tags_as_text": String("to_review"), "task_completed": Number(0), "task_count": Number(16), "updated_at": String("2022/04/26 18:41:15 +1000"), "user_count": Number(1), "user_updated_at": String("2022/04/26 18:41:15 +1000")})
// Serialize is only here for tests - is there a way around this?

#[derive(PartialEq, Debug, Deserialize, Serialize)]
pub struct Checklist {
    pub id: i32,
    pub name: String,
    // TODO: automatically convert to a date type of some sort
    pub updated_at: String,
    pub task_count: u16,
}
// TODO: model the rest (task contents)
#[derive(PartialEq, Debug, Serialize, Deserialize)]
pub struct Task {
    pub content: String,
    pub id: i32,
}

#[derive(PartialEq, Debug, Clone)]
pub enum CheckvistError {
    // TODO: may remove this variant
    AuthTokenFailure { message: String },
    UnknownError { message: String },
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
        Self::UnknownError {
            message: original.kind().to_string(),
        }
    }
}

#[derive(Debug)]
struct Client {}
impl Client {
    pub fn new() -> Client {
        Client {}
    }
}

#[derive(Debug)]
pub struct ChecklistClient {
    client: Client,
    base_url: Url,
    api_token: String,
}

impl ChecklistClient {
    pub fn new(base_url: String, api_token: String) -> Self {
        Self {
            client: Client::new(),
            base_url: Url::parse(&base_url).unwrap(),
            api_token,
        }
    }

    pub fn get_list(&self, list_id: u32) -> Result<Checklist, CheckvistError> {
        let url = self
            .base_url
            .join(&format!("/checklists/{}.json", list_id))
            .expect("Error creating url (should never happen)");

        #[derive(Deserialize)]
        #[serde(untagged)]
        enum ApiResponse {
            ValidList(Checklist),
            JsonError { message: String },
        }
        let checklist: ApiResponse = ureq::get(url.as_str())
            .set("X-Client-token", &self.api_token)
            .call()?
            .into_json()?;

        match checklist {
            ApiResponse::ValidList(returned_list) => Ok(returned_list),
            // Q: should we parse out known errors here? (eg auth). But it's all based on an (only assumed stable) 'message' string
            ApiResponse::JsonError { message } => Err(CheckvistError::UnknownError { message }),
        }
    }
}

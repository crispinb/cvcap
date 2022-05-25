#![allow(dead_code)]
use reqwest::Client;
use serde::{Deserialize, Serialize};
use url::Url;

// TODO: compile questions and ask on Rust forum
// TODO: https://crates.io/crates/secrecy for the token (wrapper type to avoid exposing during logging etc)
// TODO: investigate thiserror & anyhow
//
// curl --header "X-Client-Token: [token]" "https://checkvist.com/checklists.json"
//     .get("https://checkvist.com/checklists.json")
// current token: HRpvPJqF4uvwVR8jQ3mkiqlwCm7Y6n
// if token is bad or expired, receive: `{"message":"Unauthenticated: no valid authentication data in request"}`
// json return:
//Object({"archived": Bool(false), "created_at": String("2020/09/13 21:45:52 +0000"), "id": Number(774394), "item_count": Number(16), "markdown?": Bool(true), "name": String("devtest"), "options": Number(3), "percent_completed": Number(0.0), "public": Bool(false), "read_only": Bool(false), "related_task_ids": Null, "tags": Object({"to_review": Bool(false)}), "tags_as_text": String("to_review"), "task_completed": Number(0), "task_count": Number(16), "updated_at": String("2022/04/26 18:41:15 +1000"), "user_count": Number(1), "user_updated_at": String("2022/04/26 18:41:15 +1000")})
// Serialize is onl6y here for tests - is there a way around this?

#[derive(Debug, Deserialize, Serialize)]
pub struct Checklist {
    pub id: i32,
    pub name: String,
    // TODO: can we automatically convert to a date type of some sort?
    pub updated_at: String,
    pub task_count: u16,
}
// TODO: model the rest (task contents)
#[derive(Debug, Serialize, Deserialize)]
pub struct Task {
    pub content: String,
    pub id: i32,
    // TODO: model the rest
}

#[derive(Debug, Clone)]
pub struct Error {
    pub message: String,
    // TODO: find out why this gets a type error, or find some other way to
    //       propagate the specifics
    // TODO: make this optional?
    // pub innerError: Box<dyn std::error::Error + Send + Sync + 'static>
}

impl From<reqwest::Error> for Error {
    fn from(_: reqwest::Error) -> Self {
        Error {
            message: "fark!".to_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum CheckvistError {
    AuthTokenFailure(Error),
    UnknownError(Error),
    JsonDeserializationFailure,
}

impl From<reqwest::Error> for CheckvistError {
    fn from(original: reqwest::Error) -> Self {
        CheckvistError::UnknownError(Error {
            message: original.to_string(),
        })
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
            // TAG error_handling: how  best to hande a ParseError here
            base_url: Url::parse(&base_url).unwrap(),
            api_token,
        }
    }

    pub async fn get_list(&self, list_id: i32) -> Result<Checklist, CheckvistError> {
        let url = self
            .base_url
            .join(&format!("/checklists/{}.json", list_id))
            .unwrap();

        // Question: how to view intermediate values when debugging here?
        // They're unintelligible in the debugger.
        // println'ing often difficult becauuse of move semantics,
        // futures, etc
        let list: serde_json::Value = self
            .client
            .get(url)
            .header("X-Client-Token", &self.api_token)
            .send()
            .await?
            .json()
            .await?;

        // TODO: refactor this hellscape
        if let Some(json_object) = list.as_object() {
            if json_object.contains_key("message") {
                Err(CheckvistError::AuthTokenFailure(Error {
                    message: "auth token invalid".to_string(),
                }))
            } else {
                let checklist = serde_json::from_value(list).unwrap();
                Ok(checklist)
            }
        } else {
            Err(CheckvistError::JsonDeserializationFailure)
        }
    }

    // and/or a method to get the checklist and all embedded tasks
    pub async fn get_all_tasks(&self, list_id: i32) -> Result<Vec<Task>, reqwest::Error> {
        let url = self
            .base_url
            .join(&format!("/checklists/{}/tasks.json", list_id))
            .unwrap();
        let tasks = self
            .client
            .get(url)
            .header("X-Client-token", &self.api_token)
            .send()
            .await?
            .json()
            .await?;

        Ok(tasks)
    }
}

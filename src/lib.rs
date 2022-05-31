#![allow(dead_code)]
use reqwest::Client;
use serde::{Deserialize, Serialize};
use url::Url;

// curl --header "X-Client-Token: [token]" "https://checkvist.com/checklists.json"
//     .get("https://checkvist.com/checklists.json")
// current token: HRpvPJqF4uvwVR8jQ3mkiqlwCm7Y6n
// if token is bad or expired, receive: `{"message":"Unauthenticated: no valid authentication data in request"}`
// json return:
//Object({"archived": Bool(false), "created_at": String("2020/09/13 21:45:52 +0000"), "id": Number(774394), "item_count": Number(16), "markdown?": Bool(true), "name": String("devtest"), "options": Number(3), "percent_completed": Number(0.0), "public": Bool(false), "read_only": Bool(false), "related_task_ids": Null, "tags": Object({"to_review": Bool(false)}), "tags_as_text": String("to_review"), "task_completed": Number(0), "task_count": Number(16), "updated_at": String("2022/04/26 18:41:15 +1000"), "user_count": Number(1), "user_updated_at": String("2022/04/26 18:41:15 +1000")})
// Serialize is only here for tests - is there a way around this?

#[derive(Debug, Deserialize, Serialize)]
pub struct Checklist {
    pub id: i32,
    pub name: String,
    // TODO: automatically convert to a date type of some sort
    pub updated_at: String,
    pub task_count: u16,
}
// TODO: model the rest (task contents)
#[derive(Debug, Serialize, Deserialize)]
pub struct Task {
    pub content: String,
    pub id: i32,
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

// This is CheckvistError's internal type
// Problems:
// - seems a bit verbose. Is there something we can do inline in CheckVistError instead?
// - we really want variants - eg. a message for errors we generate, but an inner error
//   for (eg.) reqwest::error
#[derive(Debug, Clone)]
pub struct Error {
    pub message: String,
}

// TODO: build this error properly
impl From<reqwest::Error> for Error {
    fn from(_: reqwest::Error) -> Self {
        Error {
            message: "TODO".to_string(),
        }
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
                // TODO: remove unwrap
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

    pub async fn add_task(
        &self,
        list_id: i32,
        task: &TempTaskForAdding,
    ) -> Result<Task, reqwest::Error> {
        let url = self
            .base_url
            .join(&format!("/checklists/{}/tasks.json", list_id))
            .unwrap();
        println!("about to add task {:?}, via url {}", task, url);
        let returned_task: Task = self
            .client
            .post(url)
            .header("X-Client-token", &self.api_token)
            .header("Content-Type", "application/json")
            .json(task)
            .send()
            .await?
            .json()
            .await?;
        // .text()
        // .await
        // .unwrap();

        // println!("got: {:?}", returned_task);

        // Ok(Task {id: 1, content: "arked mate".into()})
        Ok(returned_task)
    }
}

// TODO: decide on how to model internal VS external task
// not sure yet whether Task should be modelled with optional fields,
// or differnet structs or in and output?
#[derive(Debug, Serialize, Clone)]
pub struct TempTaskForAdding {
    pub content: String,
    pub position: u16,
}

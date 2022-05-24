#![allow(dead_code)]
use reqwest::Client;
use serde::{Deserialize, Serialize};
use url::Url;

// TOdO: investigate error handling for when we get an error with a 200 status code. eg. when
// unauthenticated. We want in this case to NOT decode json to  our struct. Or to decode to a
// different struct handling the known errors?
//
// TODO: debugger (explore, see if will be useful) [ a debugger is a bit like a REPL for a compiled language]
// TODO: https://crates.io/crates/secrecy for the token (wrapper type to avoid exposing during logging etc)

// curl --header "X-Client-Token: [token]" "https://checkvist.com/checklists.json"
//     .get("https://checkvist.com/checklists.json")
// current token: HRpvPJqF4uvwVR8jQ3mkiqlwCm7Y6n
// if token is bad or expired, receive: `{"message":"Unauthenticated: no valid authentication data in request"}`
// json return:
//Object({"archived": Bool(false), "created_at": String("2020/09/13 21:45:52 +0000"), "id": Number(774394), "item_count": Number(16), "markdown?": Bool(true), "name": String("devtest"), "options": Number(3), "percent_completed": Number(0.0), "public": Bool(false), "read_only": Bool(false), "related_task_ids": Null, "tags": Object({"to_review": Bool(false)}), "tags_as_text": String("to_review"), "task_completed": Number(0), "task_count": Number(16), "updated_at": String("2022/04/26 18:41:15 +1000"), "user_count": Number(1), "user_updated_at": String("2022/04/26 18:41:15 +1000")})

// Serialize is onl6y here for tests - is there a way around this?
#[derive(Debug, Deserialize, Serialize)]
pub struct Checklist {
    id: i32,
    name: String,
    // TODO: can we automatically convert to a date type of some sort?
    updated_at: String,
    task_count: u16,
    // TODO: model the rest (task contents)
}
#[derive(Debug, Serialize, Deserialize)]
pub struct Task {
    id: i32,
    content: String,
    // TODO: model the rest
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
            // TAG error_handling: how  best to hande a ParseError here/
            base_url: Url::parse(&base_url).unwrap(),
            api_token,
        }
    }

    // TAG error_handling:  how best to handle?
    // boxing errors anonymously here.
    // a library, custom error type, what?
    // String here is a stopgap (forcing use of .unwrap)
    pub async fn get_list(&self, list_id: i32) -> Result<Checklist, reqwest::Error> {
        // pub async fn get_list(&self, list_id: i32) -> Result<Checklist, Box<dyn std::error::Error>> {
        let url = self
            .base_url
            .join(&format!("/checklists/{}.json", list_id))
            .unwrap();

        let list = self
            .client
            .get(url)
            .header("X-Client-Token", &self.api_token)
            .send()
            .await?
            .json()
            .await?;

        Ok(list)
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

#[tokio::main]
async fn main() {
    let client = ChecklistClient::new(
        "https://checkvist.com/".into(),
        "HRpvPJqF4uvwVR8jQ3mkiqlwCm7Y6n".into(),
    );
    let list = client.get_list(774394).await.unwrap();
    println!("list details: {:?}", list);

    // let tasks = client.get_all_tasks(774394).await.unwrap();
    // println!("tasks: {:?}", tasks);
}

// TODO: move tests when split to lib
#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::{header, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn test_get_list() {
        let mock_server = MockServer::start().await;

        let expected_checklist = Checklist {
            id: 1,
            name: "list1".to_string(),
            updated_at: "a date".to_string(),
            task_count: 1,
        };

        // The problem with this matching approach is that if it doesn't match, we get a failure in
        // the client, which isn't as informative as an explicit verification would be .expect and
        // then the (automatic) .verify() only checks the number of invocations. I'd rather have a
        // more general match, and separate verification criteria for the specifics of the call.
        Mock::given(method("GET"))
            .and(header("X-Client-Token", "token"))
            .and(path("/checklists/1.json"))
            .respond_with(ResponseTemplate::new(200).set_body_json(expected_checklist))
            .expect(1)
            .mount(&mock_server)
            .await;

        let client = ChecklistClient::new(mock_server.uri(), "token".to_string());
        let list = client.get_list(1).await.unwrap();
        assert_eq!(list.id, 1);
        assert_eq!(list.name, "list1");
        assert_eq!(list.updated_at, "a date");
        assert_eq!(list.task_count, 1);
        // not needed - called automatically
        // mock_server.verify().await;
    }

    #[tokio::test]
    async fn test_test_list_tasks() {
        let mock_server = MockServer::start().await;
        let task = Task {
            id: 1,
            content: "content".to_string(),
        };
        Mock::given(method("GET"))
            .and(header("X-Client-Token", "token"))
            .and(path("/checklists/1/tasks.json"))
            .respond_with(ResponseTemplate::new(200).set_body_json(vec![task]))
            .expect(1)
            .mount(&mock_server)
            .await;

        let client = ChecklistClient::new(mock_server.uri(), "token".to_string());
        let tasks = client.get_all_tasks(1).await.unwrap();
        assert_eq!(tasks.len(), 1);
    }

    // TODO: test for unauthenticated
}

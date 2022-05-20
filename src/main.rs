#![allow(dead_code)]
use reqwest::Client;
use serde::{Deserialize, Serialize};
use url::Url;

// TODO: task method
// TODO: embed tasks in checklist automatically?
// TODO: debugger

// curl --header "X-Client-Token: [token]" "https://checkvist.com/checklists.json"
//     .get("https://checkvist.com/checklists.json")
// current token: aPzOkkaU8ObYKFoMLYHrOlEgOjTytW
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

    pub async fn get_list(&self, list_id: i32) -> Result<Checklist, reqwest::Error> {
        // TAG error_handling: how to handle ParseError here?
        let url = self
            .base_url
            .join(&format!("/checklists/{}.json", list_id))
            .unwrap();
        let list: Checklist = self
            .client
            .get(url)
            .header("X-Client-Token", &self.api_token)
            .send()
            .await?
            .json()
            .await?;

        Ok(list)
    }
}

#[tokio::main]
async fn main() -> Result<(), reqwest::Error> {
    let client = ChecklistClient::new(
        "https://checkvist.com/".into(),
        "aPzOkkaU8ObYKFoMLYHrOlEgOjTytW".into(),
    );
    let list = client.get_list(774394).await?;

    println!("list details: {:?}", list);
    Ok(())
}

// TODO: move tests when split to lib
#[cfg(test)]
mod tests {
    use super::{Checklist, ChecklistClient};
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
}

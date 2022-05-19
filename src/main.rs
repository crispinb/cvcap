#![allow(dead_code)]
#![allow(unused_variables)]
use reqwest::Client;
use serde::Deserialize;

// curl --header "X-Client-Token: [token]" "https://checkvist.com/checklists.json"
//     .get("https://checkvist.com/checklists.json")
// current token: aPzOkkaU8ObYKFoMLYHrOlEgOjTytW
// json return:
//Object({"archived": Bool(false), "created_at": String("2020/09/13 21:45:52 +0000"), "id": Number(774394), "item_count": Number(16), "markdown?": Bool(true), "name": String("devtest"), "options": Number(3), "percent_completed": Number(0.0), "public": Bool(false), "read_only": Bool(false), "related_task_ids": Null, "tags": Object({"to_review": Bool(false)}), "tags_as_text": String("to_review"), "task_completed": Number(0), "task_count": Number(16), "updated_at": String("2022/04/26 18:41:15 +1000"), "user_count": Number(1), "user_updated_at": String("2022/04/26 18:41:15 +1000")})

#[derive(Debug, Deserialize)]
pub struct Checklist {
    id: i32,
    name: String,
}

#[derive(Debug)]
pub struct ChecklistClient {
    client: Client,
    base_url: String,
    api_token: String,
}

impl ChecklistClient {
    pub fn new(base_url: String, api_token: String) -> Self {
        Self {
            client: Client::new(),
            base_url,
            api_token,
        }
    }

    pub async fn get_list(&self, list_id: i32, token: &str) -> Result<(), String> {
        todo!()
    }
}

// TODO: get a basic async request working
// TODO: Replace raw json with a Checklist struct
// TODO: add a list-mutating call
// TODO: unit tests
// TODO: debugger
#[tokio::main]
async fn main() -> Result<(), reqwest::Error> {
    // let c = ChecklistClient::new(
    //     "https://checkvist.com/checklists.json".into(),
    //     "aPzOkkaU8ObYKFoMLYHrOlEgOjTytW".into(),
    // );
    // c.get_list(9191, "fark!");
    // println!("I have a {:?}!", c);
    
    println!("about to call ...");
    
    let list_json: serde_json::Value = 
        reqwest::Client::new()
        .get("https://checkvist.com/checklists/774394.json")
        .header("X-Client-Token", "aPzOkkaU8ObYKFoMLYHrOlEgOjTytW")
        .send()
        .await?
        .json()
        .await?;

        println!("got: {:?}", list_json);
        
        Ok(())

}

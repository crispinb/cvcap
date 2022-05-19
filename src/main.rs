#![allow(dead_code)]
use std::io::Read;
use serde::Deserialize;
use url::{ParseError, Url};
// curl --header "X-Client-Token: [token]" "https://checkvist.com/checklists.json"

#[derive(Deserialize, Debug)]
struct Checklist {
    id: u32,
    name: String,
    // tasks
}

// TODO: Replace raw json with a Checklist struct
// TODO: add a list-mutating call
// TODO: unit tests
// TODO: debugger
fn main() {
    // let client = reqwest::blocking::Client::new();

    // let mut res = client
    //     .get("https://checkvist.com/checklists.json")
    //     .header("X-Client-Token", "urcajLCgk2p9aM5xIu6kGRjBDl2Byo")
    //     .send()
    //     .unwrap();
    // let mut body = String::new();
    // res.read_to_string(&mut body);
    // println!("response: {:?}", res);
    // println!("body: \n{:?}", body);

    // let parsed: Value = serde_json::from_str(&body).unwrap();
    // println!("json: \n{:?}", parsed);

    println!("{}", get_list(774394, "aPzOkkaU8ObYKFoMLYHrOlEgOjTytW").unwrap());
}

fn get_list(list_id: u32, token: &str) -> Result<String, ParseError> {
    let base = Url::parse("https://checkvist.com/checklists/")?;
    let url = base.join(&(format!("{}{}", list_id, ".json")))?;
    let client = reqwest::blocking::Client::new();
    let mut res = client
        .get(url)
        .header("X-Client-Token", token)
        .send()
        // can't use ? here because reqwest::Error isn't compatible with
        // ParseError.
        // What's the typical way to deal with this? Handle errors and
        // consolidate internally to the function? Or return some wider error type?
        .unwrap();

    let mut body = String::new();
   // TOdO: deal with possible Error value of res 
    res.read_to_string(&mut body);
    Ok(body)

    // let json: Value = serde_json::from_str(&body).unwrap();
    // Ok(json)
}

// async approach
// #[tokio::main]
// async fn main() -> Result<(), Error> {
//   let body = reqwest::get("https://checkvist.com/checklists.json")
//   .await?
//   .text()
//   .await?;
// println!("Body returned! \n {:?}", &body);
//

//   Ok(())
// }

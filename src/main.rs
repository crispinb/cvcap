use checkvistcli::ChecklistClient;

#[tokio::main]
async fn main() {
    let client = ChecklistClient::new(
        "https://checkvist.com/".into(),
        "bad_token".into(),
        // "HRpvPJqF4uvwVR8jQ3mkiqlwCm7Y6n".into(),
    );
    let list = client.get_list(774394).await.unwrap();
    println!("list details: {:?}", list);

    // let tasks = client.get_all_tasks(774394).await.unwrap();
    // println!("tasks: {:?}", tasks);
}
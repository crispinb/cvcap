use chrono::prelude::*;
use cvapi::{Checklist, CHECKVIST_DATE_FORMAT};
use serde_json::{json, Value};

#[test]
fn checklist_de() {
    let expected = get_list();
    let json = format!(
        r#"
     {{
        "id": 1,
        "name": "checklist",
        "task_count": 2,
        "updated_at":  "{}"
     }} 
    "#,
        expected.updated_at.format(CHECKVIST_DATE_FORMAT)
    );
    let actual: Checklist = serde_json::from_str(&json).unwrap();

    assert_eq!(actual, expected);
}

#[test]
fn checklist_se() {
    let list = get_list();
    // comparing json values to avoid trivial string diffs (order, whitespace etc)
    let list_date_checkvist_format = format!("{}", list.updated_at.format(CHECKVIST_DATE_FORMAT));
    let expected = json!({
        "id": 1,
        "name": "checklist",
        "task_count": 2,
        "updated_at": list_date_checkvist_format });

    let actual: Value = serde_json::from_str(&serde_json::to_string(&list).unwrap()).unwrap();

    assert_eq!(actual, expected);
}

fn get_list() -> Checklist {
    Checklist {
        id: 1,
        name: "checklist".into(),
        task_count: 2,
        updated_at: list_date(),
    }
}

fn list_date() -> DateTime<FixedOffset> {
    let list_date: DateTime<FixedOffset> = "2022-09-01T10:58:52+10:00".parse().unwrap();
    list_date
}

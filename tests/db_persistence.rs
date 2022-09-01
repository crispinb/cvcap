#![allow(unused, unused_imports)]
use cvapi::Checklist;
use rusqlite::{Connection, Result};
use chrono::prelude::*;
use cvapi::sqlite_store::SqliteStore;

#[test]
fn simplest_persist_retrieve() {
    let now: DateTime<FixedOffset> = Local::now().try_into().unwrap();
    let list = Checklist {
        id: 1,
        name: "test".into(),
        updated_at: now, 
        task_count: 42,
    };
    let store = SqliteStore ::init().unwrap();

    let number_stored = store.save(&list).unwrap();
    let retrieved_lists: Vec<Checklist> = store.fetch_all().unwrap();

    assert_eq!(number_stored, 1);
    assert_eq!(retrieved_lists.len(), 1);
    assert_eq!(retrieved_lists[0], list);
}

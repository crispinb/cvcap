#![allow(unused, unused_imports)]
use cvapi::Checklist;
use rusqlite::{Connection, Result};
use cvapi::sqlite_store::SqliteStore;

#[test]
fn simplest_persist_retrieve_api() {
    let list = Checklist {
        id: 1,
        name: "test".into(),
        updated_at: "never".into(),
        task_count: 42,
    };

    let store = SqliteStore ::init().unwrap();
    let number_stored = store.save(&list).unwrap();

    let retrieved_lists: Vec<Checklist> = store.fetch_all().unwrap();

    assert_eq!(1, number_stored);
    assert_eq!(1, retrieved_lists.len());
    assert_eq!(list, retrieved_lists[0]);
}

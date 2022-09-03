#![allow(unused, unused_imports)]
mod utils;
use utils::*;

use chrono::prelude::*;
use cvapi::sqlite_store::SqliteStore;
use cvapi::{Checklist, Task};
use rusqlite::{Connection, Result};

#[test]
fn save_and_retrieve_one_list() {
    let list = Checklist {
        id: 1,
        name: "test".into(),
        updated_at: now(),
        task_count: 42,
    };
    let store = SqliteStore::init().unwrap();

    let number_stored = store.save_list(&list).unwrap();
    let retrieved_lists: Vec<Checklist> = store.fetch_all_lists().unwrap();

    assert_eq!(number_stored, 1);
    assert_eq!(retrieved_lists.len(), 1);
    assert_eq!(retrieved_lists[0], list);
}

#[test]
fn save_and_retrieve_multiple_lists() {
    let lists_json = std::fs::read_to_string("tests/data/checklists.json").unwrap();
    let lists: Vec<Checklist> = serde_json::from_str(&lists_json).unwrap();
    let store = SqliteStore::init().unwrap();

    let number_saved = store.save_lists(&lists).unwrap();
    let retrieved_lists = store.fetch_all_lists().unwrap();

    assert_eq!(retrieved_lists.len(), lists.len());
    assert_eq!(number_saved, lists.len());
}

#[test]
fn save_and_retrieve_one_task() {
    let task = Task {
        id: Some(1),
        content: "content".into(),
        position: 1, // TODO: add date
    };
    let store = SqliteStore::init().unwrap();

    let saved_count = store.save_task(&task).unwrap();
    let retrieved = store.fetch_all_tasks().unwrap();

    assert_eq!(saved_count, 1);
    assert_eq!(retrieved[0], task);
}



// init with filepath - create schema if not exists

// fast large insertions https://github.com/avinassh/fast-sqlite3-inserts/blob/009694f/src/bin/basic_batched.rs
// upshot: use prepared statements & batch

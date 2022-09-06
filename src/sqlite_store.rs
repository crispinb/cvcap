use std::path::Path;

use crate::{Checklist, Task};
use rusqlite::{Connection, Result};
pub struct SqliteStore {
    conn: Connection,
}

// fast large insertions https://github.com/avinassh/fast-sqlite3-inserts/blob/009694f/src/bin/basic_batched.rs
// upshot: use prepared statements & batch


impl SqliteStore {
    pub fn init_in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        migrate(&conn)?;

        Ok(SqliteStore { conn })
    }

    pub fn init_with_file(path: &Path) -> Result<Self> {
        let conn = Connection::open(path)?;
        migrate(&conn)?;

        Ok(SqliteStore { conn })
    }

    // TODO: was there struct assistance available in rustqlite or addition?
    pub fn save_list(&self, list: &Checklist) -> Result<usize> {
        let rowcount = self.conn.execute(
            r#"
            INSERT INTO checklist (checkvist_id, name, updated_at, task_count)
            VALUES (?, ?, ?, ?)
        "#,
            (list.id, &list.name, &list.updated_at, list.task_count),
        )?;

        Ok(rowcount)
    }

    pub fn save_lists(&self, lists: &Vec<Checklist>) -> Result<usize> {
        // No benefit of a transaction here I don't think.
        let mut stmt = self.conn.prepare_cached(
            r#"
            INSERT INTO checklist (checkvist_id, name, updated_at, task_count)
            VALUES (?, ?, ?, ?)
        "#,
        )?;
        for list in lists {
            stmt.execute((list.id, &list.name, list.updated_at, list.task_count))?;
        }

        Ok(lists.len())
    }

    pub fn fetch_all_lists(&self) -> Result<Vec<Checklist>> {
        let mut stmt = self
            .conn
            .prepare(r#" SELECT checkvist_id, name, updated_at, task_count from checklist "#)?;
        let lists_iter = stmt.query_map([], |row| {
            Ok(Checklist {
                id: row.get(0)?,
                name: row.get(1)?,
                updated_at: row.get(2)?,
                task_count: row.get(3)?,
            })
        })?;

        lists_iter.collect()
    }

    pub fn save_task(&self, task: &Task) -> Result<usize> {
        let sql = r#"
        INSERT INTO task (checkvist_id, content, position)
        VALUES (?,?,?)
        "#;
        let mut stmt = self.conn.prepare_cached(sql)?;

        stmt.execute((task.id, &task.content, task.position))
    }

    pub fn fetch_all_tasks(&self) -> Result<Vec<Task>> {
        let sql = r#"SELECT checkvist_id, content, position FROM task"#;
        let mut stmt = self.conn.prepare_cached(sql)?;
        let tasks_iter = stmt.query_map([], |row| {
            Ok(Task {
                id: row.get(0)?,
                content: row.get(1)?,
                position: row.get(2)?,
            })
        })?;

        tasks_iter.collect()
    }

    /// temporary for cli testing
    pub fn temp_delete_lists(&self) -> Result<()> {
        let sql = "delete from checklist";
        self.conn.execute(sql, [])?;
        Ok(())
    }
}

// TODO: scaffold possible future migrations. Table with current schema version will do for now
fn migrate(conn: &Connection) -> Result<(), rusqlite::Error> {
    conn.execute(
        r#"
        CREATE TABLE IF NOT EXISTS checklist (
            id INTEGER PRIMARY KEY,
            checkvist_id INTEGER UNIQUE,
            name TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            task_count INTEGER NOT NULL
        )
        STRICT
        "#,
        (),
    )?;
    conn.execute(
        r#"
        CREATE TABLE IF NOT EXISTS task (
            id INTEGER PRIMARY KEY,
            checkvist_id INTEGER UNIQUE,
            content TEXT NOT NULL,
            position INTEGER NOT NULL
        )
        STRICT
        "#,
        (),
    )?;
    Ok(())
}

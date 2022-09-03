use crate::{Checklist, Task};
use rusqlite::{Connection, Result};
pub struct SqliteStore {
    conn: Connection,
}
 
impl SqliteStore {
    pub fn init() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        // TODO: move into a migration method when move to file-based db
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
}

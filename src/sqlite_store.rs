use crate::Checklist;
use rusqlite::{Connection, Result};
pub struct SqliteStore {
    conn: Connection,
}

impl SqliteStore {
    pub fn init() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        // TODO: move into some kind of migration method when move to file-based db
        // TODO: types: checkvist api, struct, sqlite, rusqlite
        conn.execute(
            r#"
        CREATE TABLE checklist (
            id INTEGER PRIMARY KEY,
            checkvist_id INTEGER,
            name TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            task_count INTEGER NOT NULL
        )
        STRICT
        "#,
            (),
        )?;

        Ok(SqliteStore { conn })
    }

    // TODO: was there some sort of struct assistance available in rustqlite or addition?
    pub fn save(&self, list: &Checklist) -> Result<usize> {
        let rowcount = self.conn.execute(
            r#"
            INSERT INTO checklist (checkvist_id, name, updated_at, task_count)
            VALUES (?1, ?2, ?3, ?4)
        "#,
            (list.id, &list.name, &list.updated_at, list.task_count),
        )?;

        Ok(rowcount)
    }

    pub fn fetch_all(&self) -> Result<Vec<Checklist>> {
        let mut select_lists = self
            .conn
            .prepare(r#" SELECT checkvist_id, name, updated_at, task_count from checklist "#)?;
        let lists_iter = select_lists.query_map([], |row| {
            Ok(Checklist {
                id: row.get(0)?,
                name: row.get(1)?,
                updated_at: row.get(2)?,
                task_count: row.get(3)?,
            })
        })?;

        lists_iter.collect()
    }
}

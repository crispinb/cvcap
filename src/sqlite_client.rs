use crate::{sqlite_store::SqliteStore, ApiClient, Checklist, Task, CheckvistClient, CheckvistError};

/// Alternative to CheckvistClient that performs all ops via SqliteStore

pub struct SqliteClient {
    api_client: ApiClient,
    store: SqliteStore,
}

impl SqliteClient {
    pub fn new(checkvist_client: ApiClient, store: SqliteStore) -> Self {
        SqliteClient {
            api_client: checkvist_client,
            store,
        }
    }

    /// Placeholder for cli testing. Not yet a real sync 
    pub fn sync_lists(&self) -> Result<(), CheckvistError> {
        let lists = self.api_client.get_lists()?;
        self.store.temp_delete_lists()?;
        self.store.save_lists(&lists)?;

        Ok(())
    }

}

impl CheckvistClient for SqliteClient {
    fn get_lists(&self) -> Result<Vec<Checklist>, CheckvistError> {
        let lists = self.store.fetch_all_lists()?;

        Ok(lists)
    }

    // TODO: implement delegated methods
    fn get_list(&self, list_id: u32) -> Result<Checklist, CheckvistError> {
        self.api_client.get_list(list_id)
    }

    fn add_list(&self, list_name: &str) -> Result<Checklist, CheckvistError> {
        self.api_client.add_list(list_name)
    }

    fn get_tasks(&self, list_id: u32) -> Result<Vec<Task>, CheckvistError> {
        self.api_client.get_tasks(list_id)
    }

    fn add_task(&self, list_id: u32, task: &Task) -> Result<Task, CheckvistError> {
        self.api_client.add_task(list_id, task)
    }
}

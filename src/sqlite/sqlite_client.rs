use crate::{sqlite::SqliteStore, ApiClient, Checklist, CheckvistClient, Result, Task};

/// Alternative to CheckvistClient that performs all ops via SqliteStore

pub struct SqliteSyncClient {
    api_client: ApiClient,
    store: SqliteStore,
}

impl SqliteSyncClient {
    pub fn new(checkvist_client: ApiClient, store: SqliteStore) -> Self {
        SqliteSyncClient {
            api_client: checkvist_client,
            store,
        }
    }

    /// Placeholde for cli testing. Not yet a real sync
    pub fn sync_lists(&self) -> Result<()> {
        let lists = self.api_client.get_lists()?;
        self.store.temp_delete_lists()?;
        self.store.save_lists(&lists)?;

        Ok(())
    }

    pub fn sync_tasks(&self, list_id: u32) -> Result<()> {
        let tasks = self.api_client.get_tasks(list_id)?;
        self.store.temp_delete_tasks(list_id)?;
        self.store.save_tasks(&tasks)?;

        Ok(())
    }
}

impl CheckvistClient for SqliteSyncClient {
    fn get_lists(&self) -> Result<Vec<Checklist>> {
        let lists = self.store.fetch_all_lists()?;

        Ok(lists)
    }

    // TODO: implement delegated methods
    fn get_list(&self, list_id: u32) -> Result<Checklist> {
        self.api_client.get_list(list_id)
    }

    fn add_list(&self, list_name: &str) -> Result<Checklist> {
        self.api_client.add_list(list_name)
    }

    fn get_tasks(&self, list_id: u32) -> Result<Vec<Task>> {
        // TODO: limit to list
        let tasks = self.store.fetch_tasks_for_list(list_id)?;
        Ok(tasks)
    }

    fn add_task(&self, list_id: u32, task: &Task) -> Result<Task> {
        self.api_client.add_task(list_id, task)
    }
}

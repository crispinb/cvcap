use crate::{sqlite_store::SqliteStore, Checklist, CheckvistClient, CheckvistError};

/// Alternative to CheckvistClient that performs all ops via SqliteStore

pub struct PersistentCheckvistClient {
    checkvist_client: CheckvistClient,
    store: SqliteStore,
}

impl PersistentCheckvistClient {
    pub fn new(checkvist_client: CheckvistClient, store: SqliteStore) -> Self {
        PersistentCheckvistClient {
            checkvist_client,
            store,
        }
    }

    pub fn sync_lists(&self) -> Result<(), CheckvistError> {
        let lists = self.checkvist_client.get_lists()?;
        self.store.save_lists(&lists)?;

        Ok(())
    }

    pub fn fetch_all_lists(&self) -> Result<Vec<Checklist>, CheckvistError> {
        let lists = self.store.fetch_all_lists()?;

        Ok(lists)
    }
}

use crate::Task;

// TODO: temp pending our own Error type
use anyhow::anyhow;
use anyhow::Error as TempError;

type Result<T> = std::result::Result<T, Error>;

type Error = TempError;

fn get_local_tasks(list_id: u32) -> Result<Vec<Task>> {
    Err(anyhow!("temp"))
}

fn get_syncstate_tasks(list_id: u32) -> Result<Vec<Task>> {
    Err(anyhow!("temp"))
}

fn get_remote_tasks(list_id: u32) -> Result<Vec<Task>> {
    Err(anyhow!("temp"))
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]

    // no, the task syncer should receive tasks (as sets?) and do the reconciliation.
    fn get_all_sync_sets_no_tasks() {
        assert_eq!(get_local_tasks(1).unwrap().len(), 0);
        assert_eq!(get_remote_tasks(1).unwrap().len(), 0);
        assert_eq!(get_syncstate_tasks(1).unwrap().len(), 0);
    }
}

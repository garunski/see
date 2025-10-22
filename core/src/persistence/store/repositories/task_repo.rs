// Task repository - task execution operations

use crate::errors::CoreError;
use crate::persistence::models::TaskExecution;
use crate::persistence::store::db_ops::DatabaseOperations;
use crate::persistence::store::keys::task_key;
use crate::persistence::store::serialization::serialize;
use redb::{Table, TableDefinition};
use tracing::trace;

const TASKS_TABLE: &str = "tasks";
const TASKS_DEF: TableDefinition<&str, &[u8]> = TableDefinition::new(TASKS_TABLE);

/// Repository for task execution operations
#[derive(Debug)]
pub struct TaskRepository {
    db_ops: DatabaseOperations,
}

impl TaskRepository {
    /// Create a new TaskRepository
    pub fn new(db_ops: DatabaseOperations) -> Self {
        Self { db_ops }
    }

    /// Save a task execution
    pub async fn save_task(&self, task: &TaskExecution) -> Result<(), CoreError> {
        let task = task.clone();
        let key = task_key(&task.execution_id, &task.task_id);

        trace!("Saving task execution");
        self.db_ops
            .execute_write(move |db| {
                let write_txn = db.begin_write()?;
                {
                    let mut tasks_table: Table<&str, &[u8]> = write_txn.open_table(TASKS_DEF)?;
                    let serialized = serialize(&task)?;
                    tasks_table.insert(key.as_str(), serialized.as_slice())?;
                }
                write_txn.commit()?;
                Ok(())
            })
            .await
    }
}

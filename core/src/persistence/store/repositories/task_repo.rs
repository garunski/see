// Task repository - task execution operations

use crate::errors::CoreError;
use crate::persistence::models::TaskExecution;
use crate::persistence::store::db_ops::DatabaseOperations;
use crate::persistence::store::keys::task_key;
use crate::persistence::store::serialization::{deserialize, serialize};
use redb::{ReadOnlyTable, ReadableTable, Table, TableDefinition};
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

    /// Get all tasks for a specific execution
    #[allow(dead_code)]
    pub async fn get_tasks_for_execution(
        &self,
        execution_id: &str,
    ) -> Result<Vec<TaskExecution>, CoreError> {
        let execution_id = execution_id.to_string();
        let task_prefix = format!("task:{}:", execution_id);

        self.db_ops
            .execute_read(move |db| {
                let read_txn = db.begin_read()?;
                let tasks_table: ReadOnlyTable<&str, &[u8]> = read_txn.open_table(TASKS_DEF)?;

                let mut tasks = Vec::new();
                for item in tasks_table.iter()? {
                    let (key, value) = item?;
                    let key_str = key.value();
                    if key_str.starts_with(&task_prefix) {
                        let task_exec: TaskExecution = deserialize(value.value())?;
                        tasks.push(task_exec);
                    }
                }

                Ok(tasks)
            })
            .await
    }

    /// Delete all tasks for a specific execution
    #[allow(dead_code)]
    pub async fn delete_tasks_for_execution(&self, execution_id: &str) -> Result<(), CoreError> {
        let execution_id = execution_id.to_string();
        let task_prefix = format!("task:{}:", execution_id);

        self.db_ops
            .execute_write(move |db| {
                let write_txn = db.begin_write()?;
                {
                    let mut tasks_table: Table<&str, &[u8]> = write_txn.open_table(TASKS_DEF)?;
                    let mut keys_to_delete = Vec::new();

                    for item in tasks_table.iter()? {
                        let (key, _) = item?;
                        if key.value().starts_with(&task_prefix) {
                            keys_to_delete.push(key.value().to_string());
                        }
                    }

                    for key in keys_to_delete {
                        tasks_table.remove(key.as_str())?;
                    }
                }
                write_txn.commit()?;
                Ok(())
            })
            .await
    }
}

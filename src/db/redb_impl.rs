use super::{AuditStore, WorkflowExecution, WorkflowExecutionSummary};
use bincode;
use redb::{Database, ReadableTable, Table, ReadOnlyTable};
use std::error::Error;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::task;

const EXECUTIONS_TABLE: &str = "executions";
const EXECUTION_IDS_TABLE: &str = "execution_ids";

/// Redb-backed implementation of AuditStore
#[derive(Debug)]
pub struct RedbAuditStore {
    db: Arc<Database>,
}

impl RedbAuditStore {
    /// Create a new RedbAuditStore with database at the specified path
    pub fn new(db_path: PathBuf) -> Result<Self, Box<dyn Error>> {
        // Ensure parent directory exists
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let db = Database::create(db_path)?;
        
        // Define tables
        let write_txn = db.begin_write()?;
        {
            let _executions_table: Table<&str, &[u8]> = write_txn.open_table(redb::TableDefinition::new(EXECUTIONS_TABLE))?;
            let _execution_ids_table: Table<&str, &str> = write_txn.open_table(redb::TableDefinition::new(EXECUTION_IDS_TABLE))?;
        }
        write_txn.commit()?;

        Ok(Self {
            db: Arc::new(db),
        })
    }

    /// Get the default database path in user's home directory
    pub fn default_path() -> Result<PathBuf, Box<dyn Error>> {
        let home_dir = dirs::home_dir()
            .ok_or("Could not find home directory")?;
        Ok(home_dir.join(".see").join("audit.redb"))
    }

    /// Create a new RedbAuditStore with default path
    pub fn new_default() -> Result<Self, Box<dyn Error>> {
        Self::new(Self::default_path()?)
    }
}

impl AuditStore for RedbAuditStore {
    fn save_workflow_execution(&self, execution: &WorkflowExecution) -> Result<String, Box<dyn Error + Send + Sync>> {
        let db = Arc::clone(&self.db);
        let execution = execution.clone();
        let id = execution.id.clone();

        // Use spawn_blocking since redb is synchronous
        task::block_in_place(|| {
            let write_txn = db.begin_write()?;
            {
                let mut executions_table: Table<&str, &[u8]> = write_txn.open_table(redb::TableDefinition::new(EXECUTIONS_TABLE))?;
                let mut execution_ids_table: Table<&str, &str> = write_txn.open_table(redb::TableDefinition::new(EXECUTION_IDS_TABLE))?;

                // Serialize execution data
                let serialized = bincode::serialize(&execution)?;

                // Store execution
                executions_table.insert(id.as_str(), serialized.as_slice())?;

                // Store in execution IDs table for listing (ordered by timestamp)
                let timestamp_key = format!("{}:{}", execution.timestamp, id);
                execution_ids_table.insert(timestamp_key.as_str(), id.as_str())?;
            }
            write_txn.commit()?;
            Ok(id)
        })
    }

    fn get_workflow_execution(&self, id: &str) -> Result<WorkflowExecution, Box<dyn Error + Send + Sync>> {
        let db = Arc::clone(&self.db);
        let id = id.to_string();

        task::block_in_place(|| {
            let read_txn = db.begin_read()?;
            let executions_table: ReadOnlyTable<&str, &[u8]> = read_txn.open_table(redb::TableDefinition::new(EXECUTIONS_TABLE))?;

                if let Some(serialized) = executions_table.get(&*id)? {
                let execution: WorkflowExecution = bincode::deserialize(&serialized.value())?;
                Ok(execution)
            } else {
                Err(format!("Workflow execution with id '{}' not found", id).into())
            }
        })
    }

    fn list_workflow_executions(&self, limit: usize) -> Result<Vec<WorkflowExecutionSummary>, Box<dyn Error + Send + Sync>> {
        let db = Arc::clone(&self.db);

        task::block_in_place(|| {
            let read_txn = db.begin_read()?;
            let execution_ids_table: ReadOnlyTable<&str, &str> = read_txn.open_table(redb::TableDefinition::new(EXECUTION_IDS_TABLE))?;
            let executions_table: ReadOnlyTable<&str, &[u8]> = read_txn.open_table(redb::TableDefinition::new(EXECUTIONS_TABLE))?;

            let mut summaries = Vec::new();
            let mut count = 0;

            // Iterate through execution IDs in reverse order (newest first)
            for item in execution_ids_table.iter()?.rev() {
                if count >= limit {
                    break;
                }

                let (_, id_value) = item?;
                let id: &str = id_value.value();

                if let Some(serialized) = executions_table.get(&*id)? {
                    let execution: WorkflowExecution = bincode::deserialize(&serialized.value())?;
                    summaries.push(WorkflowExecutionSummary {
                        id: execution.id,
                        workflow_name: execution.workflow_name,
                        timestamp: execution.timestamp,
                        success: execution.success,
                        task_count: execution.tasks.len(),
                    });
                    count += 1;
                }
            }

            Ok(summaries)
        })
    }

    fn delete_workflow_execution(&self, id: &str) -> Result<(), Box<dyn Error + Send + Sync>> {
        let db = Arc::clone(&self.db);
        let id = id.to_string();

        task::block_in_place(|| {
            let write_txn = db.begin_write()?;
            {
                let mut executions_table: Table<&str, &[u8]> = write_txn.open_table(redb::TableDefinition::new(EXECUTIONS_TABLE))?;
                let mut execution_ids_table: Table<&str, &str> = write_txn.open_table(redb::TableDefinition::new(EXECUTION_IDS_TABLE))?;

                // First get the execution to find its timestamp for the ID table
                let timestamp_key = {
                    let execution_data = executions_table.get(&*id)?;
                    if let Some(serialized) = execution_data {
                        let execution: WorkflowExecution = bincode::deserialize(&serialized.value())?;
                        format!("{}:{}", execution.timestamp, id)
                    } else {
                        return Err(format!("Workflow execution with id '{}' not found", id).into());
                    }
                };

                // Remove from both tables
                executions_table.remove(id.as_str())?;
                execution_ids_table.remove(timestamp_key.as_str())?;
            }
            write_txn.commit()?;
            Ok(())
        })
    }
}

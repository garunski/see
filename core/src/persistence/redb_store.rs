#![allow(clippy::result_large_err)]
use crate::errors::CoreError;
use crate::persistence::models::{WorkflowExecution, WorkflowExecutionSummary};
use async_trait::async_trait;
use bincode;
use redb::{Database, ReadOnlyTable, ReadableTable, Table};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::task;

const EXECUTIONS_TABLE: &str = "executions";
const EXECUTION_IDS_TABLE: &str = "execution_ids";

#[derive(Debug)]
pub struct RedbStore {
    db: Arc<Database>,
}

impl RedbStore {
    pub fn new(db_path: PathBuf) -> Result<Self, CoreError> {
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let db = Database::create(db_path)?;
        let write_txn = db.begin_write()?;
        {
            let _executions_table: Table<&str, &[u8]> =
                write_txn.open_table(redb::TableDefinition::new(EXECUTIONS_TABLE))?;
            let _execution_ids_table: Table<&str, &str> =
                write_txn.open_table(redb::TableDefinition::new(EXECUTION_IDS_TABLE))?;
        }
        write_txn.commit()?;
        Ok(Self { db: Arc::new(db) })
    }

    pub fn default_path() -> Result<PathBuf, CoreError> {
        let home_dir = dirs::home_dir()
            .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, "home dir"))?;
        Ok(home_dir.join(".see").join("audit.redb"))
    }

    pub fn new_default() -> Result<Self, CoreError> {
        Self::new(Self::default_path()?)
    }
}

#[async_trait]
pub trait AuditStore: Send + Sync {
    async fn save_workflow_execution(
        &self,
        execution: &WorkflowExecution,
    ) -> Result<String, CoreError>;
    async fn get_workflow_execution(&self, id: &str) -> Result<WorkflowExecution, CoreError>;
    async fn list_workflow_executions(
        &self,
        limit: usize,
    ) -> Result<Vec<WorkflowExecutionSummary>, CoreError>;
    async fn delete_workflow_execution(&self, id: &str) -> Result<(), CoreError>;
}

#[async_trait]
impl AuditStore for RedbStore {
    async fn save_workflow_execution(
        &self,
        execution: &WorkflowExecution,
    ) -> Result<String, CoreError> {
        let db = Arc::clone(&self.db);
        let execution = execution.clone();
        let id = execution.id.clone();
        task::spawn_blocking(move || -> Result<String, CoreError> {
            let write_txn = db.begin_write()?;
            {
                let mut executions_table: Table<&str, &[u8]> =
                    write_txn.open_table(redb::TableDefinition::new(EXECUTIONS_TABLE))?;
                let mut execution_ids_table: Table<&str, &str> =
                    write_txn.open_table(redb::TableDefinition::new(EXECUTION_IDS_TABLE))?;
                let serialized = bincode::serialize(&execution)
                    .map_err(|e| CoreError::Dataflow(e.to_string()))?;
                executions_table.insert(id.as_str(), serialized.as_slice())?;
                let timestamp_key = format!("{}:{}", execution.timestamp, id);
                execution_ids_table.insert(timestamp_key.as_str(), id.as_str())?;
            }
            write_txn.commit()?;
            Ok(id)
        })
        .await
        .map_err(|e| CoreError::Dataflow(format!("task join error: {}", e)))?
    }

    async fn get_workflow_execution(&self, id: &str) -> Result<WorkflowExecution, CoreError> {
        let db = Arc::clone(&self.db);
        let id = id.to_string();
        task::spawn_blocking(move || -> Result<WorkflowExecution, CoreError> {
            let read_txn = db.begin_read()?;
            let executions_table: ReadOnlyTable<&str, &[u8]> =
                read_txn.open_table(redb::TableDefinition::new(EXECUTIONS_TABLE))?;
            if let Some(serialized) = executions_table.get(id.as_str())? {
                let execution: WorkflowExecution = bincode::deserialize(serialized.value())
                    .map_err(|e| CoreError::Dataflow(e.to_string()))?;
                Ok(execution)
            } else {
                Err(CoreError::Dataflow(format!(
                    "Workflow execution with id '{}' not found",
                    id
                )))
            }
        })
        .await
        .map_err(|e| CoreError::Dataflow(format!("task join error: {}", e)))?
    }

    async fn list_workflow_executions(
        &self,
        limit: usize,
    ) -> Result<Vec<WorkflowExecutionSummary>, CoreError> {
        let db = Arc::clone(&self.db);
        task::spawn_blocking(
            move || -> Result<Vec<WorkflowExecutionSummary>, CoreError> {
                let read_txn = db.begin_read()?;
                let execution_ids_table: ReadOnlyTable<&str, &str> =
                    read_txn.open_table(redb::TableDefinition::new(EXECUTION_IDS_TABLE))?;
                let executions_table: ReadOnlyTable<&str, &[u8]> =
                    read_txn.open_table(redb::TableDefinition::new(EXECUTIONS_TABLE))?;
                let mut summaries = Vec::new();
                let mut count = 0;
                for item in execution_ids_table.iter()?.rev() {
                    if count >= limit {
                        break;
                    }
                    let (_, id_value) = item?;
                    let id: &str = id_value.value();
                    if let Some(serialized) = executions_table.get(id)? {
                        let execution: WorkflowExecution = bincode::deserialize(serialized.value())
                            .map_err(|e| CoreError::Dataflow(e.to_string()))?;
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
            },
        )
        .await
        .map_err(|e| CoreError::Dataflow(format!("task join error: {}", e)))?
    }

    async fn delete_workflow_execution(&self, id: &str) -> Result<(), CoreError> {
        let db = Arc::clone(&self.db);
        let id = id.to_string();
        task::spawn_blocking(move || -> Result<(), CoreError> {
            let write_txn = db.begin_write()?;
            {
                let mut executions_table: Table<&str, &[u8]> =
                    write_txn.open_table(redb::TableDefinition::new(EXECUTIONS_TABLE))?;
                let mut execution_ids_table: Table<&str, &str> =
                    write_txn.open_table(redb::TableDefinition::new(EXECUTION_IDS_TABLE))?;
                let timestamp_key = {
                    let execution_data = executions_table.get(id.as_str())?;
                    if let Some(serialized) = execution_data {
                        let execution: WorkflowExecution = bincode::deserialize(serialized.value())
                            .map_err(|e| CoreError::Dataflow(e.to_string()))?;
                        format!("{}:{}", execution.timestamp, id)
                    } else {
                        return Err(CoreError::Dataflow(format!(
                            "Workflow execution with id '{}' not found",
                            id
                        )));
                    }
                };
                executions_table.remove(id.as_str())?;
                execution_ids_table.remove(timestamp_key.as_str())?;
            }
            write_txn.commit()?;
            Ok(())
        })
        .await
        .map_err(|e| CoreError::Dataflow(format!("task join error: {}", e)))?
    }
}

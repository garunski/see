// Workflow repository - workflow execution & metadata operations

use crate::errors::CoreError;
use crate::persistence::models::{
    TaskExecution, WorkflowExecution, WorkflowExecutionSummary, WorkflowMetadata, WorkflowStatus,
};
use crate::persistence::store::db_ops::DatabaseOperations;
use crate::persistence::store::keys::{
    execution_timestamp_key, task_prefix, workflow_metadata_key,
};
use crate::persistence::store::serialization::{deserialize, serialize};
use crate::types::TaskInfo;
use redb::{ReadOnlyTable, ReadableTable, Table, TableDefinition};
use std::collections::HashMap;
use tracing::{debug, instrument, trace};

const EXECUTIONS_TABLE: &str = "executions";
const EXECUTION_IDS_TABLE: &str = "execution_ids";
const TASKS_TABLE: &str = "tasks";

const EXECUTIONS_DEF: TableDefinition<&str, &[u8]> = TableDefinition::new(EXECUTIONS_TABLE);
const EXECUTION_IDS_DEF: TableDefinition<&str, &str> = TableDefinition::new(EXECUTION_IDS_TABLE);
const TASKS_DEF: TableDefinition<&str, &[u8]> = TableDefinition::new(TASKS_TABLE);

/// Repository for workflow execution and metadata operations
#[derive(Debug)]
pub struct WorkflowRepository {
    db_ops: DatabaseOperations,
}

impl WorkflowRepository {
    /// Create a new WorkflowRepository
    pub fn new(db_ops: DatabaseOperations) -> Self {
        Self { db_ops }
    }

    /// Save a workflow execution
    #[instrument(skip(self, execution), fields(execution_id = %execution.id))]
    pub async fn save_execution(&self, execution: &WorkflowExecution) -> Result<String, CoreError> {
        let execution = execution.clone();
        let id = execution.id.clone();

        trace!("Entering save_workflow_execution");
        let db = self.db_ops.database().clone();
        self.db_ops
            .execute_write_with_retry(move || {
                let execution = execution.clone();
                let id = id.clone();
                tokio::task::block_in_place(|| {
                    trace!("Beginning write transaction");
                    let write_txn = db.begin_write()?;
                    {
                        trace!("Opening tables");
                        let mut executions_table: Table<&str, &[u8]> =
                            write_txn.open_table(EXECUTIONS_DEF)?;
                        let mut execution_ids_table: Table<&str, &str> =
                            write_txn.open_table(EXECUTION_IDS_DEF)?;
                        let serialized = serialize(&execution)?;
                        debug!(serialized_size = serialized.len(), "Serialized execution");
                        executions_table.insert(id.as_str(), serialized.as_slice())?;
                        trace!("Inserted into executions table");
                        let timestamp_key = execution_timestamp_key(&execution.timestamp, &id);
                        execution_ids_table.insert(timestamp_key.as_str(), id.as_str())?;
                        trace!("Inserted into execution_ids table");
                    }
                    trace!("Committing transaction");
                    write_txn.commit()?;
                    trace!("Write transaction committed successfully");
                    Ok(id)
                })
            })
            .await
    }

    /// Get a workflow execution by ID
    pub async fn get_execution(&self, id: &str) -> Result<WorkflowExecution, CoreError> {
        let id = id.to_string();
        self.db_ops
            .execute_read(move |db| {
                let read_txn = db.begin_read()?;
                let executions_table: ReadOnlyTable<&str, &[u8]> =
                    read_txn.open_table(EXECUTIONS_DEF)?;
                if let Some(serialized) = executions_table.get(id.as_str())? {
                    let execution: WorkflowExecution = deserialize(serialized.value())?;
                    Ok(execution)
                } else {
                    Err(CoreError::Dataflow(format!(
                        "Workflow execution with id '{}' not found",
                        id
                    )))
                }
            })
            .await
    }

    /// List workflow executions with pagination
    pub async fn list_executions(
        &self,
        limit: usize,
    ) -> Result<Vec<WorkflowExecutionSummary>, CoreError> {
        self.db_ops
            .execute_read(move |db| {
                let read_txn = db.begin_read()?;
                let execution_ids_table: ReadOnlyTable<&str, &str> =
                    read_txn.open_table(EXECUTION_IDS_DEF)?;
                let executions_table: ReadOnlyTable<&str, &[u8]> =
                    read_txn.open_table(EXECUTIONS_DEF)?;
                let mut summaries = Vec::new();
                let mut count = 0;
                for item in execution_ids_table.iter()?.rev() {
                    if count >= limit {
                        break;
                    }
                    let (_, id_value) = item?;
                    let id: &str = id_value.value();
                    if let Some(serialized) = executions_table.get(id)? {
                        let execution: WorkflowExecution = deserialize(serialized.value())?;
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
            .await
    }

    /// Delete a workflow execution and all associated data
    pub async fn delete_execution(&self, id: &str) -> Result<(), CoreError> {
        let id = id.to_string();
        self.db_ops
            .execute_write(move |db| {
                let write_txn = db.begin_write()?;
                {
                    let mut executions_table: Table<&str, &[u8]> =
                        write_txn.open_table(EXECUTIONS_DEF)?;
                    let mut execution_ids_table: Table<&str, &str> =
                        write_txn.open_table(EXECUTION_IDS_DEF)?;
                    let mut tasks_table: Table<&str, &[u8]> = write_txn.open_table(TASKS_DEF)?;

                    let timestamp_key = {
                        let execution_data = executions_table.get(id.as_str())?;
                        if let Some(serialized) = execution_data {
                            let execution: WorkflowExecution = deserialize(serialized.value())?;
                            execution_timestamp_key(&execution.timestamp, &id)
                        } else {
                            return Err(CoreError::Dataflow(format!(
                                "Workflow execution with id '{}' not found",
                                id
                            )));
                        }
                    };

                    executions_table.remove(id.as_str())?;
                    execution_ids_table.remove(timestamp_key.as_str())?;

                    let metadata_key = workflow_metadata_key(&id);
                    executions_table.remove(metadata_key.as_str())?;

                    let task_prefix = task_prefix(&id);
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

    /// Save workflow metadata
    #[instrument(skip(self, metadata), fields(metadata_id = %metadata.id, status = ?metadata.status))]
    pub async fn save_metadata(&self, metadata: &WorkflowMetadata) -> Result<(), CoreError> {
        let metadata = metadata.clone();
        let key = workflow_metadata_key(&metadata.id);

        trace!("Saving workflow metadata");
        self.db_ops
            .execute_write(move |db| {
                let write_txn = db.begin_write()?;
                {
                    let mut workflows_table: Table<&str, &[u8]> =
                        write_txn.open_table(EXECUTIONS_DEF)?;
                    let serialized = serialize(&metadata)?;
                    workflows_table.insert(key.as_str(), serialized.as_slice())?;
                }
                write_txn.commit()?;
                Ok(())
            })
            .await
    }

    /// Get workflow metadata by ID
    #[allow(dead_code)]
    pub async fn get_metadata(&self, id: &str) -> Result<WorkflowMetadata, CoreError> {
        let id = id.to_string();
        self.db_ops
            .execute_read(move |db| {
                let read_txn = db.begin_read()?;
                let workflows_table: ReadOnlyTable<&str, &[u8]> =
                    read_txn.open_table(EXECUTIONS_DEF)?;
                let workflow_key = workflow_metadata_key(&id);
                if let Some(serialized) = workflows_table.get(workflow_key.as_str())? {
                    let metadata: WorkflowMetadata = deserialize(serialized.value())?;
                    Ok(metadata)
                } else {
                    Err(CoreError::Dataflow(format!("Workflow {} not found", id)))
                }
            })
            .await
    }

    /// List workflow metadata with pagination
    pub async fn list_metadata(&self, limit: usize) -> Result<Vec<WorkflowMetadata>, CoreError> {
        self.db_ops
            .execute_read(move |db| {
                let read_txn = db.begin_read()?;
                let workflows_table: ReadOnlyTable<&str, &[u8]> =
                    read_txn.open_table(EXECUTIONS_DEF)?;

                let mut metadata_list = Vec::new();
                let mut count = 0;

                for item in workflows_table.iter()? {
                    if count >= limit {
                        break;
                    }

                    let (key, value) = item?;
                    if key.value().starts_with("workflow:") {
                        let metadata: WorkflowMetadata = deserialize(value.value())?;
                        metadata_list.push(metadata);
                        count += 1;
                    }
                }

                metadata_list.sort_by(|a, b| b.start_timestamp.cmp(&a.start_timestamp));

                Ok(metadata_list)
            })
            .await
    }

    /// Get workflow with tasks reconstructed from metadata and task executions
    pub async fn get_with_tasks(&self, execution_id: &str) -> Result<WorkflowExecution, CoreError> {
        let execution_id = execution_id.to_string();
        self.db_ops
            .execute_read(move |db| {
                let read_txn = db.begin_read()?;

                let workflows_table: ReadOnlyTable<&str, &[u8]> =
                    read_txn.open_table(EXECUTIONS_DEF)?;
                let workflow_key = workflow_metadata_key(&execution_id);
                let metadata: WorkflowMetadata =
                    if let Some(serialized) = workflows_table.get(workflow_key.as_str())? {
                        deserialize(serialized.value())?
                    } else {
                        return Err(CoreError::Dataflow(format!(
                            "Workflow {} not found",
                            execution_id
                        )));
                    };

                let tasks_table: ReadOnlyTable<&str, &[u8]> = read_txn.open_table(TASKS_DEF)?;
                let task_prefix = task_prefix(&execution_id);

                let mut tasks = Vec::new();
                let mut per_task_logs = HashMap::new();

                for item in tasks_table.iter()? {
                    let (key, value) = item?;
                    let key_str = key.value();
                    if key_str.starts_with(&task_prefix) {
                        let task_exec: TaskExecution = deserialize(value.value())?;

                        tasks.push(TaskInfo {
                            id: task_exec.task_id.clone(),
                            name: task_exec.task_name.clone(),
                            status: task_exec.status,
                        });

                        per_task_logs.insert(task_exec.task_id.clone(), task_exec.logs.clone());
                    }
                }

                let ordered_tasks = if !metadata.task_ids.is_empty() {
                    let mut ordered_tasks = Vec::new();
                    for task_id in &metadata.task_ids {
                        if let Some(task) = tasks.iter().find(|t| &t.id == task_id) {
                            ordered_tasks.push(task.clone());
                        }
                    }
                    ordered_tasks
                } else {
                    tasks
                };

                Ok(WorkflowExecution {
                    id: metadata.id,
                    workflow_name: metadata.workflow_name,
                    timestamp: metadata.start_timestamp,
                    success: metadata.status == WorkflowStatus::Complete,
                    tasks: ordered_tasks,
                    audit_trail: vec![],
                    per_task_logs,
                    errors: if metadata.status == WorkflowStatus::Failed {
                        vec!["Workflow failed".to_string()]
                    } else {
                        vec![]
                    },
                })
            })
            .await
    }

    /// Delete workflow metadata and all associated tasks
    pub async fn delete_metadata_and_tasks(&self, execution_id: &str) -> Result<(), CoreError> {
        let execution_id = execution_id.to_string();
        self.db_ops
            .execute_write(move |db| {
                let write_txn = db.begin_write()?;
                {
                    let mut executions_table: Table<&str, &[u8]> =
                        write_txn.open_table(EXECUTIONS_DEF)?;
                    let mut tasks_table: Table<&str, &[u8]> = write_txn.open_table(TASKS_DEF)?;

                    let metadata_key = workflow_metadata_key(&execution_id);
                    executions_table.remove(metadata_key.as_str())?;

                    let task_prefix = task_prefix(&execution_id);
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

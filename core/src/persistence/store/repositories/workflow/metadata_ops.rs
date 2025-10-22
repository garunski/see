// Workflow metadata operations

use crate::errors::CoreError;
use crate::persistence::models::{
    TaskExecution, WorkflowExecution, WorkflowMetadata, WorkflowStatus,
};
use crate::persistence::store::db_ops::DatabaseOperations;
use crate::persistence::store::keys::{task_prefix, workflow_metadata_key};
use crate::persistence::store::serialization::deserialize;
use crate::types::TaskInfo;
use redb::{ReadOnlyTable, ReadableTable, Table};
use std::collections::HashMap;
use tracing::{instrument, trace};

use super::table_operations::{TableOperations, WorkflowTableOps};
use super::types::{TableContext, EXECUTIONS_DEF, TASKS_DEF};

/// Operations for workflow metadata
#[derive(Debug)]
pub struct MetadataOperations {
    db_ops: DatabaseOperations,
}

impl MetadataOperations {
    pub fn new(db_ops: DatabaseOperations) -> Self {
        Self { db_ops }
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
                    workflows_table.insert_serialized(&key, &metadata)?;
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
                if let Some(metadata) =
                    workflows_table.get_by_key::<WorkflowMetadata>(&workflow_key)?
                {
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

                let metadata: WorkflowMetadata = if let Some(meta) =
                    workflows_table.get_by_key::<WorkflowMetadata>(&workflow_key)?
                {
                    meta
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
                    let executions_table: Table<&str, &[u8]> =
                        write_txn.open_table(EXECUTIONS_DEF)?;
                    let tasks_table: Table<&str, &[u8]> = write_txn.open_table(TASKS_DEF)?;

                    let mut tables = TableContext {
                        executions_table,
                        execution_ids_table: write_txn
                            .open_table(super::types::EXECUTION_IDS_DEF)?,
                        tasks_table,
                    };

                    WorkflowTableOps::delete_metadata_and_tasks(&mut tables, &execution_id)?;
                }
                write_txn.commit()?;
                Ok(())
            })
            .await
    }
}

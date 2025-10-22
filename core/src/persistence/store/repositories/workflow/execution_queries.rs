// Dedicated execution query service for task reconstruction

use crate::errors::CoreError;
use crate::persistence::models::{WorkflowExecution, WorkflowMetadata, WorkflowStatus};
use crate::persistence::store::db_ops::DatabaseOperations;
use crate::persistence::store::keys::{task_prefix, workflow_metadata_key};
use crate::persistence::store::serialization::deserialize;
use crate::types::TaskInfo;
use redb::{ReadOnlyTable, ReadableTable};
use std::collections::HashMap;
use tracing::instrument;

use super::table_operations::TableOperations;

use super::types::{EXECUTIONS_DEF, TASKS_DEF};

/// Result of task reconstruction containing tasks and their logs
type TaskReconstructionResult = (Vec<TaskInfo>, HashMap<String, Vec<String>>);

/// Service for querying workflow executions with task reconstruction
#[derive(Debug)]
pub struct ExecutionQueryService {
    db_ops: DatabaseOperations,
}

impl ExecutionQueryService {
    /// Create a new execution query service
    pub fn new(db_ops: DatabaseOperations) -> Self {
        Self { db_ops }
    }

    /// Get workflow execution with tasks reconstructed from metadata and task executions
    #[instrument(skip(self))]
    pub async fn get_with_tasks(
        &self,
        execution_id: &str,
    ) -> Result<Option<WorkflowExecution>, CoreError> {
        let db_ops = self.db_ops.clone();
        let execution_id = execution_id.to_string();
        db_ops
            .execute_read(move |db| {
                let read_txn = db.begin_read()?;

                let workflows_table: ReadOnlyTable<&str, &[u8]> =
                    read_txn.open_table(EXECUTIONS_DEF)?;
                let workflow_key = workflow_metadata_key(&execution_id);

                // Get metadata
                let metadata: WorkflowMetadata = if let Some(meta) =
                    workflows_table.get_by_key::<WorkflowMetadata>(&workflow_key)?
                {
                    meta
                } else {
                    return Ok(None);
                };

                // Get tasks
                let tasks_table: ReadOnlyTable<&str, &[u8]> = read_txn.open_table(TASKS_DEF)?;
                let task_prefix = task_prefix(&execution_id);

                let (tasks, per_task_logs) = Self::reconstruct_tasks(&tasks_table, &task_prefix)?;

                // Order tasks according to metadata task_ids
                let ordered_tasks = Self::order_tasks(&tasks, &metadata.task_ids);

                // Build execution
                let execution = WorkflowExecution {
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
                };

                Ok(Some(execution))
            })
            .await
    }

    /// Reconstruct tasks from task executions in the database
    fn reconstruct_tasks(
        tasks_table: &ReadOnlyTable<&str, &[u8]>,
        task_prefix: &str,
    ) -> Result<TaskReconstructionResult, CoreError> {
        let mut tasks = Vec::new();
        let mut per_task_logs = HashMap::new();

        for item in tasks_table.iter()? {
            let (key, value) = item?;
            let key_str = key.value();
            if key_str.starts_with(task_prefix) {
                let task_exec: crate::persistence::models::TaskExecution =
                    deserialize(value.value())?;

                tasks.push(TaskInfo {
                    id: task_exec.task_id.clone(),
                    name: task_exec.task_name.clone(),
                    status: task_exec.status,
                });

                per_task_logs.insert(task_exec.task_id.clone(), task_exec.logs.clone());
            }
        }

        Ok((tasks, per_task_logs))
    }

    /// Order tasks according to the task_ids in metadata
    fn order_tasks(tasks: &[TaskInfo], task_ids: &[String]) -> Vec<TaskInfo> {
        if task_ids.is_empty() {
            return tasks.to_vec();
        }

        let mut ordered_tasks = Vec::new();
        for task_id in task_ids {
            if let Some(task) = tasks.iter().find(|t| &t.id == task_id) {
                ordered_tasks.push(task.clone());
            }
        }
        ordered_tasks
    }
}

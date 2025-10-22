// Query builder for workflow operations

use crate::errors::CoreError;
use crate::persistence::models::{WorkflowExecution, WorkflowMetadata, WorkflowStatus};
use crate::persistence::store::db_ops::DatabaseOperations;
use crate::persistence::store::keys::{task_prefix, workflow_metadata_key};
use crate::persistence::store::serialization::deserialize;
use crate::types::TaskInfo;
use chrono::{DateTime, Utc};
use redb::{ReadOnlyTable, ReadableTable};
use std::collections::HashMap;
use std::time::SystemTime;
use tracing::instrument;

use super::table_operations::TableOperations;
use super::types::{EXECUTIONS_DEF, TASKS_DEF};

/// Query options for workflow metadata
#[derive(Debug, Clone)]
pub struct WorkflowQueryOptions {
    pub limit: Option<usize>,
    pub offset: Option<usize>,
    pub status_filter: Option<WorkflowStatus>,
    pub workflow_name_filter: Option<String>,
    pub start_time_after: Option<SystemTime>,
    pub start_time_before: Option<SystemTime>,
    pub sort_by: WorkflowSortField,
    pub sort_order: SortOrder,
}

impl Default for WorkflowQueryOptions {
    fn default() -> Self {
        Self {
            limit: Some(50),
            offset: Some(0),
            status_filter: None,
            workflow_name_filter: None,
            start_time_after: None,
            start_time_before: None,
            sort_by: WorkflowSortField::StartTime,
            sort_order: SortOrder::Descending,
        }
    }
}

#[derive(Debug, Clone)]
pub enum WorkflowSortField {
    StartTime,
}

#[derive(Debug, Clone)]
pub enum SortOrder {
    Descending,
}

/// Query builder for workflow operations
#[derive(Debug)]
pub struct WorkflowQueryBuilder {
    db_ops: DatabaseOperations,
    pub options: WorkflowQueryOptions,
}

impl WorkflowQueryBuilder {
    pub fn new(db_ops: DatabaseOperations) -> Self {
        Self {
            db_ops,
            options: WorkflowQueryOptions::default(),
        }
    }

    /// Execute the query and return workflow metadata
    #[instrument(skip(self))]
    pub async fn execute_metadata(&self) -> Result<Vec<WorkflowMetadata>, CoreError> {
        let options = self.options.clone();
        self.db_ops
            .execute_read(move |db| {
                let read_txn = db.begin_read()?;
                let workflows_table: ReadOnlyTable<&str, &[u8]> =
                    read_txn.open_table(EXECUTIONS_DEF)?;

                let mut metadata_list = Vec::new();
                let mut count = 0;
                let offset = options.offset.unwrap_or(0);

                for item in workflows_table.iter()? {
                    if count >= offset + options.limit.unwrap_or(usize::MAX) {
                        break;
                    }

                    let (key, value) = item?;
                    if key.value().starts_with("workflow:") {
                        let metadata: WorkflowMetadata = deserialize(value.value())?;

                        // Apply filters
                        if let Some(status) = &options.status_filter {
                            if &metadata.status != status {
                                continue;
                            }
                        }

                        if let Some(name_filter) = &options.workflow_name_filter {
                            if !metadata.workflow_name.contains(name_filter) {
                                continue;
                            }
                        }

                        if let Some(after) = &options.start_time_after {
                            if let Ok(metadata_time) =
                                metadata.start_timestamp.parse::<DateTime<Utc>>()
                            {
                                let metadata_system_time: SystemTime = metadata_time.into();
                                if metadata_system_time < *after {
                                    continue;
                                }
                            }
                        }

                        if let Some(before) = &options.start_time_before {
                            if let Ok(metadata_time) =
                                metadata.start_timestamp.parse::<DateTime<Utc>>()
                            {
                                let metadata_system_time: SystemTime = metadata_time.into();
                                if metadata_system_time > *before {
                                    continue;
                                }
                            }
                        }

                        if count >= offset {
                            metadata_list.push(metadata);
                        }
                        count += 1;
                    }
                }

                // Apply sorting
                match options.sort_by {
                    WorkflowSortField::StartTime => {
                        metadata_list.sort_by(|a, b| match options.sort_order {
                            SortOrder::Descending => b.start_timestamp.cmp(&a.start_timestamp),
                        });
                    }
                }

                Ok(metadata_list)
            })
            .await
    }

    /// Get workflow with tasks reconstructed from metadata and task executions
    #[instrument(skip(self))]
    pub async fn get_with_tasks(
        &self,
        execution_id: &str,
    ) -> Result<Option<WorkflowExecution>, CoreError> {
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
                    return Ok(None);
                };

                let tasks_table: ReadOnlyTable<&str, &[u8]> = read_txn.open_table(TASKS_DEF)?;
                let task_prefix = task_prefix(&execution_id);

                let mut tasks = Vec::new();
                let mut per_task_logs = HashMap::new();

                for item in tasks_table.iter()? {
                    let (key, value) = item?;
                    let key_str = key.value();
                    if key_str.starts_with(&task_prefix) {
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

                Ok(Some(WorkflowExecution {
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
                }))
            })
            .await
    }
}

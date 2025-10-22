// Workflow execution operations

use crate::errors::CoreError;
use crate::persistence::models::{WorkflowExecution, WorkflowExecutionSummary};
use crate::persistence::store::db_ops::DatabaseOperations;
use crate::persistence::store::serialization::serialize;
use redb::{ReadOnlyTable, ReadableTable, Table};
use tracing::{debug, instrument, trace};

use super::table_operations::{TableOperations, WorkflowTableOps};
use super::types::{TableContext, EXECUTIONS_DEF, EXECUTION_IDS_DEF};

/// Operations for workflow executions
#[derive(Debug)]
pub struct ExecutionOperations {
    db_ops: DatabaseOperations,
}

impl ExecutionOperations {
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
                        let executions_table: Table<&str, &[u8]> =
                            write_txn.open_table(EXECUTIONS_DEF)?;
                        let execution_ids_table: Table<&str, &str> =
                            write_txn.open_table(EXECUTION_IDS_DEF)?;

                        let mut tables = TableContext {
                            executions_table,
                            execution_ids_table,
                            tasks_table: write_txn.open_table(super::types::TASKS_DEF)?,
                        };

                        let serialized = serialize(&execution)?;
                        debug!(serialized_size = serialized.len(), "Serialized execution");

                        WorkflowTableOps::save_execution_with_index(&mut tables, &execution)?;
                        trace!("Inserted into both tables");
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

                if let Some(execution) = executions_table.get_by_key::<WorkflowExecution>(&id)? {
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

                    if let Some(execution) = executions_table.get_by_key::<WorkflowExecution>(id)? {
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
                    let executions_table: Table<&str, &[u8]> =
                        write_txn.open_table(EXECUTIONS_DEF)?;
                    let execution_ids_table: Table<&str, &str> =
                        write_txn.open_table(EXECUTION_IDS_DEF)?;
                    let tasks_table: Table<&str, &[u8]> =
                        write_txn.open_table(super::types::TASKS_DEF)?;

                    let mut tables = TableContext {
                        executions_table,
                        execution_ids_table,
                        tasks_table,
                    };

                    WorkflowTableOps::delete_execution_complete(&mut tables, &id)?;
                }
                write_txn.commit()?;
                Ok(())
            })
            .await
    }
}

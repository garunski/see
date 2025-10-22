// Service layer for workflow operations

use crate::errors::CoreError;
use crate::persistence::models::{WorkflowExecution, WorkflowMetadata};
use crate::persistence::store::db_ops::DatabaseOperations;
use crate::persistence::store::keys::workflow_metadata_key;
use redb::Table;
use tracing::{instrument, trace};

use super::execution_queries::ExecutionQueryService;
use super::metadata_queries::MetadataQueryService;
use super::query_types::WorkflowQueryOptions;
use super::table_operations::TableOperations;
use super::types::{TableContext, EXECUTIONS_DEF, TASKS_DEF};

/// Service for workflow metadata operations
#[derive(Debug)]
pub struct WorkflowMetadataService {
    db_ops: DatabaseOperations,
    query_service: MetadataQueryService,
}

impl WorkflowMetadataService {
    pub fn new(db_ops: DatabaseOperations) -> Self {
        let query_service = MetadataQueryService::new(db_ops.clone());
        Self {
            db_ops,
            query_service,
        }
    }

    /// Save workflow metadata with validation
    #[instrument(skip(self, metadata), fields(metadata_id = %metadata.id, status = ?metadata.status))]
    pub async fn save_metadata(&self, metadata: &WorkflowMetadata) -> Result<(), CoreError> {
        // Validate metadata
        self.validate_metadata(metadata)?;

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

    /// List workflow metadata with advanced filtering
    #[instrument(skip(self))]
    pub async fn list_metadata(
        &self,
        options: WorkflowQueryOptions,
    ) -> Result<Vec<WorkflowMetadata>, CoreError> {
        self.query_service.query_metadata(options).await
    }

    /// Delete workflow metadata and all associated tasks
    #[instrument(skip(self))]
    pub async fn delete_metadata_and_tasks(&self, execution_id: &str) -> Result<(), CoreError> {
        if execution_id.is_empty() {
            return Err(CoreError::Dataflow(
                "Execution ID cannot be empty".to_string(),
            ));
        }

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

                    super::table_operations::WorkflowTableOps::delete_metadata_and_tasks(
                        &mut tables,
                        &execution_id,
                    )?;
                }
                write_txn.commit()?;
                Ok(())
            })
            .await
    }

    /// Validate workflow metadata
    fn validate_metadata(&self, metadata: &WorkflowMetadata) -> Result<(), CoreError> {
        if metadata.id.is_empty() {
            return Err(CoreError::Dataflow(
                "Workflow ID cannot be empty".to_string(),
            ));
        }

        if metadata.workflow_name.is_empty() {
            return Err(CoreError::Dataflow(
                "Workflow name cannot be empty".to_string(),
            ));
        }

        // Validate task IDs are unique
        let mut seen_ids = std::collections::HashSet::new();
        for task_id in &metadata.task_ids {
            if !seen_ids.insert(task_id) {
                return Err(CoreError::Dataflow(format!(
                    "Duplicate task ID found: {}",
                    task_id
                )));
            }
        }

        Ok(())
    }
}

/// Service for workflow execution operations
#[derive(Debug)]
pub struct WorkflowExecutionService {
    query_service: ExecutionQueryService,
}

impl WorkflowExecutionService {
    pub fn new(db_ops: DatabaseOperations) -> Self {
        let query_service = ExecutionQueryService::new(db_ops);
        Self { query_service }
    }

    /// Get workflow with tasks reconstructed from metadata and task executions
    #[instrument(skip(self))]
    pub async fn get_with_tasks(
        &self,
        execution_id: &str,
    ) -> Result<Option<WorkflowExecution>, CoreError> {
        if execution_id.is_empty() {
            return Err(CoreError::Dataflow(
                "Execution ID cannot be empty".to_string(),
            ));
        }

        self.query_service.get_with_tasks(execution_id).await
    }
}

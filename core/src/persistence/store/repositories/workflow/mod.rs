// Workflow repository module - orchestrates execution and metadata operations

use crate::errors::CoreError;
use crate::persistence::models::{WorkflowExecution, WorkflowExecutionSummary, WorkflowMetadata};
use crate::persistence::store::db_ops::DatabaseOperations;

mod execution_ops;
mod metadata_ops;
mod table_operations;
mod types;

use execution_ops::ExecutionOperations;
use metadata_ops::MetadataOperations;

/// Repository for workflow execution and metadata operations
///
/// This orchestrator delegates to specialized operation modules:
/// - ExecutionOperations: Handles WorkflowExecution CRUD
/// - MetadataOperations: Handles WorkflowMetadata CRUD and task reconstruction
#[derive(Debug)]
pub struct WorkflowRepository {
    execution_ops: ExecutionOperations,
    metadata_ops: MetadataOperations,
}

impl WorkflowRepository {
    /// Create a new WorkflowRepository
    pub fn new(db_ops: DatabaseOperations) -> Self {
        Self {
            execution_ops: ExecutionOperations::new(db_ops.clone()),
            metadata_ops: MetadataOperations::new(db_ops),
        }
    }

    /// Save a workflow execution
    pub async fn save_execution(&self, execution: &WorkflowExecution) -> Result<String, CoreError> {
        self.execution_ops.save_execution(execution).await
    }

    /// Get a workflow execution by ID
    pub async fn get_execution(&self, id: &str) -> Result<WorkflowExecution, CoreError> {
        self.execution_ops.get_execution(id).await
    }

    /// List workflow executions with pagination
    pub async fn list_executions(
        &self,
        limit: usize,
    ) -> Result<Vec<WorkflowExecutionSummary>, CoreError> {
        self.execution_ops.list_executions(limit).await
    }

    /// Delete a workflow execution and all associated data
    pub async fn delete_execution(&self, id: &str) -> Result<(), CoreError> {
        self.execution_ops.delete_execution(id).await
    }

    /// Save workflow metadata
    pub async fn save_metadata(&self, metadata: &WorkflowMetadata) -> Result<(), CoreError> {
        self.metadata_ops.save_metadata(metadata).await
    }

    /// Get workflow metadata by ID
    #[allow(dead_code)]
    pub async fn get_metadata(&self, id: &str) -> Result<WorkflowMetadata, CoreError> {
        self.metadata_ops.get_metadata(id).await
    }

    /// List workflow metadata with pagination
    pub async fn list_metadata(&self, limit: usize) -> Result<Vec<WorkflowMetadata>, CoreError> {
        self.metadata_ops.list_metadata(limit).await
    }

    /// Get workflow with tasks reconstructed from metadata and task executions
    pub async fn get_with_tasks(&self, execution_id: &str) -> Result<WorkflowExecution, CoreError> {
        self.metadata_ops.get_with_tasks(execution_id).await
    }

    /// Delete workflow metadata and all associated tasks
    pub async fn delete_metadata_and_tasks(&self, execution_id: &str) -> Result<(), CoreError> {
        self.metadata_ops
            .delete_metadata_and_tasks(execution_id)
            .await
    }
}

// Re-export types for external use
// pub use types::{EXECUTIONS_DEF, EXECUTION_IDS_DEF, TASKS_DEF};

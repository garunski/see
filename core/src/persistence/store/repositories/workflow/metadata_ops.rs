// Modern workflow metadata operations - minimal implementation

use crate::errors::CoreError;
use crate::persistence::models::{WorkflowExecution, WorkflowMetadata};
use crate::persistence::store::db_ops::DatabaseOperations;
use tracing::instrument;

use super::services::{WorkflowExecutionService, WorkflowMetadataService};

// Re-export only what's actually used
pub use super::query_builder::WorkflowQueryOptions;

/// Modern MetadataOperations using the new service layer architecture
#[derive(Debug)]
pub struct MetadataOperations {
    metadata_service: WorkflowMetadataService,
    execution_service: WorkflowExecutionService,
}

impl MetadataOperations {
    /// Create a new MetadataOperations instance
    pub fn new(db_ops: DatabaseOperations) -> Self {
        let metadata_service = WorkflowMetadataService::new(db_ops.clone());
        let execution_service = WorkflowExecutionService::new(db_ops.clone());

        Self {
            metadata_service,
            execution_service,
        }
    }

    /// Save workflow metadata with validation
    #[instrument(skip(self, metadata), fields(metadata_id = %metadata.id, status = ?metadata.status))]
    pub async fn save_metadata(&self, metadata: &WorkflowMetadata) -> Result<(), CoreError> {
        self.metadata_service.save_metadata(metadata).await
    }

    /// List workflow metadata with simple pagination (for compatibility)
    pub async fn list_metadata_simple(
        &self,
        limit: usize,
    ) -> Result<Vec<WorkflowMetadata>, CoreError> {
        let options = WorkflowQueryOptions {
            limit: Some(limit),
            offset: Some(0),
            status_filter: None,
            workflow_name_filter: None,
            start_time_after: None,
            start_time_before: None,
            sort_by: super::query_builder::WorkflowSortField::StartTime,
            sort_order: super::query_builder::SortOrder::Descending,
        };

        self.metadata_service.list_metadata(options).await
    }

    /// Get workflow with tasks reconstructed from metadata and task executions
    pub async fn get_with_tasks(
        &self,
        execution_id: &str,
    ) -> Result<Option<WorkflowExecution>, CoreError> {
        self.execution_service.get_with_tasks(execution_id).await
    }

    /// Get workflow with tasks - throws error if not found (for compatibility with existing code)
    pub async fn get_with_tasks_required(
        &self,
        execution_id: &str,
    ) -> Result<WorkflowExecution, CoreError> {
        match self.get_with_tasks(execution_id).await? {
            Some(execution) => Ok(execution),
            None => Err(CoreError::Dataflow(format!(
                "Workflow {} not found",
                execution_id
            ))),
        }
    }

    /// Delete workflow metadata and all associated tasks
    pub async fn delete_metadata_and_tasks(&self, execution_id: &str) -> Result<(), CoreError> {
        self.metadata_service
            .delete_metadata_and_tasks(execution_id)
            .await
    }
}

pub mod models;
pub mod redb_impl;

use crate::db::models::{WorkflowExecution, WorkflowExecutionSummary};
use std::error::Error;

/// Trait for storing and retrieving workflow execution audit data
pub trait AuditStore: Send + Sync {
    /// Save a workflow execution to the store
    fn save_workflow_execution(
        &self,
        execution: &WorkflowExecution,
    ) -> Result<String, Box<dyn Error + Send + Sync>>;

    /// Get a workflow execution by ID
    fn get_workflow_execution(
        &self,
        id: &str,
    ) -> Result<WorkflowExecution, Box<dyn Error + Send + Sync>>;

    /// List workflow executions with a limit
    fn list_workflow_executions(
        &self,
        limit: usize,
    ) -> Result<Vec<WorkflowExecutionSummary>, Box<dyn Error + Send + Sync>>;

    /// Delete a workflow execution by ID
    fn delete_workflow_execution(&self, id: &str) -> Result<(), Box<dyn Error + Send + Sync>>;
}

pub use redb_impl::RedbAuditStore;

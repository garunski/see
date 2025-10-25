//! S_E_E Core - Minimal coordination layer for workflow execution
//!
//! This crate provides a minimal interface between the GUI/CLI and the engine/persistence layers.
//! It should not contain business logic - that belongs in the engine and persistence crates.

pub mod errors;
pub mod execution;
pub mod store;
pub mod task_executor;
pub mod tracing;
pub mod types;

pub use types::*;

// Re-export tracing functionality
pub use tracing::{init_tracing, TracingGuard};

// Re-export persistence functionality for convenience
pub use persistence::{initialize_database, PersistenceError};
pub use persistence::{
    Workflow, WorkflowExecution, TaskExecution, UserPrompt, AiPrompt, Setting,
    WorkflowDefinition, AppSettings, Theme, WorkflowExecutionSummary, WorkflowMetadata,
    WorkflowJson, TaskInfo, WorkflowStatus, AuditEvent, TaskStatus, AuditEntry, AuditStatus
};

// Re-export engine functionality
pub use engine::{
    execute_workflow_from_json, parse_workflow,
};

// Re-export store functionality
pub use store::{Store, PersistenceStore};

use std::sync::Arc;

/// Create a persistence store instance
pub async fn create_persistence_store() -> Result<PersistenceStore, errors::CoreError> {
    // Initialize persistence database
    initialize_database("~/.s_e_e/workflows.db").await?;
    
    // Get instance manager
    let instance_manager = persistence::get_instance_manager().await?;
    
    // Create persistence store
    Ok(PersistenceStore::new(instance_manager))
}

/// Get the global store instance (for GUI compatibility)
pub fn get_global_store() -> Result<Arc<dyn Store>, errors::CoreError> {
    // For now, create a new store instance each time
    // TODO: Implement proper global store management
    Err(errors::CoreError::Validation("Global store not implemented in new architecture".to_string()))
}

/// Execute workflow by ID (for GUI compatibility)
pub async fn execute_workflow_by_id(
    workflow_id: &str,
    _output_callback: Option<OutputCallback>,
) -> Result<WorkflowResult, errors::CoreError> {
    // TODO: Implement workflow execution by ID
    Err(errors::CoreError::Validation("Workflow execution by ID not implemented".to_string()))
}

/// Resume task (for GUI compatibility)
pub async fn resume_task(execution_id: &str, task_id: &str) -> Result<(), errors::CoreError> {
    // TODO: Implement task resumption
    Err(errors::CoreError::Validation("Task resumption not implemented".to_string()))
}

/// Resume workflow (for GUI compatibility)
pub async fn resume_workflow(execution_id: &str) -> Result<(), errors::CoreError> {
    // TODO: Implement workflow resumption
    Err(errors::CoreError::Validation("Workflow resumption not implemented".to_string()))
}

/// Pause workflow (for GUI compatibility)
pub async fn pause_workflow(execution_id: &str, task_id: &str) -> Result<(), errors::CoreError> {
    // TODO: Implement workflow pausing
    Err(errors::CoreError::Validation("Workflow pausing not implemented".to_string()))
}

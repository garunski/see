pub mod engine;
pub mod errors;
pub mod execution;
pub mod json_parser;
pub mod persistence;
pub mod store;
pub mod task_executor;
pub mod tracing;
pub mod types;
pub mod utils;

// Re-export types for backward compatibility
pub use types::*;

// Re-export tracing functionality
pub use tracing::{init_tracing, TracingGuard};

// Re-export store functionality
pub use store::get_global_store;

pub use crate::engine::execute::{execute_workflow, execute_workflow_from_content};
pub use crate::persistence::models::{
    AppSettings, TaskExecution, Theme, WorkflowDefinition, WorkflowExecution,
    WorkflowExecutionSummary, WorkflowMetadata, WorkflowStatus,
};
pub use crate::persistence::store::{AuditStore, RedbStore};

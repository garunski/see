// Core crate - Public API exports
// This crate coordinates between persistence and engine layers

pub mod api;
pub mod bridge;
pub mod errors;
pub mod logging;
pub mod store_singleton;
pub mod validation;

// Re-export persistence types
pub use persistence::{
    AppSettings,
    AuditEvent,
    AuditStatus, // Re-export from persistence (has Display trait)
    Prompt,
    Store,
    TaskExecution,
    TaskExecutionStatus, // Re-export from persistence (has as_str method)
    Theme,
    UserInputRequest,
    WorkflowDefinition,
    WorkflowExecution,
    WorkflowExecutionStatus,
    WorkflowExecutionSummary,
    WorkflowMetadata,
};

// Re-export engine types
pub use engine::{AuditEntry, EngineWorkflow, TaskInfo};

// Type alias for GUI compatibility
pub type WorkflowJson = EngineWorkflow;

// Re-export core types
pub use crate::api::{
    execute_workflow_by_id, get_pending_inputs, get_tasks_waiting_for_input, populate_initial_data,
    provide_user_input, resume_task,
};
pub use crate::bridge::WorkflowResult;
pub use crate::errors::CoreError;
pub use crate::logging::{init_tracing, TracingGuard};
pub use crate::store_singleton::{
    cleanup_test_db, get_global_store, init_global_store, init_test_store,
};

// Re-export conversion functions for GUI compatibility
pub use crate::bridge::audit::audit_event_to_entry;
pub use crate::bridge::task::task_execution_to_info;

// Re-export function types
pub use crate::bridge::OutputCallback;

// Re-export validation functions
pub use crate::validation::{validate_workflow_json, validate_workflow_json_simple};

// Core crate - Public API exports
// This crate coordinates between persistence and engine layers

pub mod api;
pub mod bridge;
pub mod errors;
pub mod logging;
pub mod store_singleton;

// Re-export persistence types
pub use persistence::{
    AppSettings,
    AuditEvent,
    AuditStatus, // Re-export from persistence (has Display trait)
    Store,
    TaskExecution,
    TaskStatus, // Re-export from persistence (has as_str method)
    Theme,
    UserPrompt,
    WorkflowDefinition,
    WorkflowExecution,
    WorkflowExecutionSummary,
    WorkflowMetadata,
    WorkflowStatus,
};

// Re-export engine types
pub use engine::{AuditEntry, EngineWorkflow, TaskInfo};

// Type alias for GUI compatibility
pub type WorkflowJson = EngineWorkflow;

// Re-export core types
pub use crate::api::{execute_workflow_by_id, resume_task};
pub use crate::bridge::WorkflowResult;
pub use crate::errors::CoreError;
pub use crate::logging::{init_tracing, TracingGuard};
pub use crate::store_singleton::{get_global_store, init_global_store};

// Re-export conversion functions for GUI compatibility
pub use crate::bridge::audit::audit_event_to_entry;
pub use crate::bridge::task::task_execution_to_info;

// Re-export function types
pub use crate::bridge::OutputCallback;

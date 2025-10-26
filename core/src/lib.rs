// Core crate - Public API exports
// This crate coordinates between persistence and engine layers

pub mod bridge;
pub mod api;
pub mod store_singleton;
pub mod errors;
pub mod logging;

// Re-export all persistence types
pub use persistence::{
    WorkflowDefinition,
    WorkflowExecution,
    WorkflowExecutionSummary,
    WorkflowMetadata,
    TaskExecution,
    UserPrompt,
    AuditEvent,
    AppSettings,
    WorkflowStatus,
    Theme,
    Store,
};

// Re-export engine types
pub use engine::{
    TaskInfo,
    TaskStatus,
    AuditStatus,
    AuditEntry,
};

// Re-export core types
pub use crate::bridge::WorkflowResult;
pub use crate::errors::CoreError;
pub use crate::api::{execute_workflow_by_id, resume_task};
pub use crate::store_singleton::{init_global_store, get_global_store};
pub use crate::logging::{init_tracing, TracingGuard};

// Re-export function types
pub use crate::bridge::OutputCallback;

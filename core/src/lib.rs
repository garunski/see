pub mod api;
pub mod bridge;
pub mod embedded_data;
pub mod errors;
pub mod logging;
pub mod store_singleton;
pub mod validation;

pub use s_e_e_persistence::{
    AppSettings, AuditEvent, AuditStatus, Prompt, Store, TaskExecution, TaskExecutionStatus, Theme,
    UserInputRequest, WorkflowDefinition, WorkflowExecution, WorkflowExecutionStatus,
    WorkflowExecutionSummary, WorkflowMetadata,
};

pub use s_e_e_engine::{AuditEntry, EngineWorkflow, TaskInfo};

pub type WorkflowJson = EngineWorkflow;

pub use crate::api::{
    delete_workflow_execution, execute_workflow_by_id, get_pending_inputs,
    get_tasks_waiting_for_input, populate_initial_data, provide_user_input,
};
pub use crate::bridge::WorkflowResult;
pub use crate::errors::CoreError;
pub use crate::logging::{init_tracing, TracingGuard};
pub use crate::store_singleton::{
    cleanup_test_db, get_global_store, init_global_store, init_test_store,
};

pub use crate::bridge::audit::audit_event_to_entry;
pub use crate::bridge::task::task_execution_to_info;

pub use crate::bridge::OutputCallback;

pub use crate::validation::{validate_workflow_json, validate_workflow_json_simple};

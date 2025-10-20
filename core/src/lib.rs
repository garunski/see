pub mod engine;
pub mod errors;
pub mod execution;
pub mod json_parser;
pub mod persistence;
pub mod task_executor;
pub mod utils;

use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;

/// Output callback type for real-time output during workflow execution
pub type OutputCallback = Arc<dyn Fn(String) + Send + Sync>;

/// Information about a single task in a workflow
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct TaskInfo {
    pub id: String,
    pub name: String,
    pub status: String, // pending, in-progress, complete, failed
}

/// Audit trail entry
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct AuditEntry {
    pub task_id: String,
    pub status: String,
    pub timestamp: String,
    pub changes_count: usize,
}

/// Workflow execution result
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct WorkflowResult {
    pub success: bool,
    pub workflow_name: String,
    pub task_count: usize,
    pub execution_id: String,
    pub tasks: Vec<TaskInfo>,
    pub final_context: Value,
    pub audit_trail: Vec<AuditEntry>,
    pub per_task_logs: HashMap<String, Vec<String>>,
    pub errors: Vec<String>,
    pub output_logs: Vec<String>,
}

pub use crate::engine::execute::execute_workflow;
pub use crate::persistence::models::{
    AppSettings, Theme, WorkflowExecution, WorkflowExecutionSummary,
};
pub use crate::persistence::redb_store::{AuditStore, RedbStore};

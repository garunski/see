//! Core types - minimal shared types for coordination

use serde_json::Value;
use std::collections::HashMap;

/// Callback for workflow output
pub type OutputCallback = std::sync::Arc<dyn Fn(String) + Send + Sync>;

/// Workflow execution result
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct WorkflowResult {
    pub success: bool,
    pub workflow_name: String,
    pub task_count: usize,
    pub execution_id: String,
    pub tasks: Vec<persistence::TaskInfo>,
    pub final_context: Value,
    pub audit_trail: Vec<persistence::AuditEvent>,
    pub per_task_logs: HashMap<String, Vec<String>>,
    pub errors: Vec<String>,
    pub output_logs: Vec<String>,
}

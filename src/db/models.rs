use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Information about a single task in a workflow
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TaskInfo {
    pub id: String,
    pub name: String,
    pub status: String, // pending, in-progress, complete, failed
}

/// Complete workflow execution record
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WorkflowExecution {
    pub id: String,
    pub workflow_name: String,
    pub timestamp: String, // ISO 8601 format
    pub success: bool,
    pub tasks: Vec<TaskInfo>,
    pub audit_trail: Vec<crate::AuditEntry>,
    pub per_task_logs: HashMap<String, Vec<String>>,
    pub errors: Vec<String>,
}

/// Summary of workflow execution for listing
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WorkflowExecutionSummary {
    pub id: String,
    pub workflow_name: String,
    pub timestamp: String,
    pub success: bool,
    pub task_count: usize,
}

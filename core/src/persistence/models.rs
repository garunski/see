use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WorkflowExecution {
    pub id: String,
    pub workflow_name: String,
    pub timestamp: String,
    pub success: bool,
    pub tasks: Vec<crate::TaskInfo>,
    pub audit_trail: Vec<crate::AuditEntry>,
    pub per_task_logs: HashMap<String, Vec<String>>,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WorkflowExecutionSummary {
    pub id: String,
    pub workflow_name: String,
    pub timestamp: String,
    pub success: bool,
    pub task_count: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum Theme {
    Light,
    Dark,
    System,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WorkflowDefinition {
    pub id: String,
    pub name: String,
    pub content: String,
    pub is_default: bool,
    pub is_edited: bool,
}

use crate::persistence::default_workflows::DefaultWorkflows;

impl WorkflowDefinition {
    /// Get all default workflow definitions
    pub fn get_default_workflows() -> Vec<Self> {
        vec![
            WorkflowDefinition {
                id: "default-echo-workflow-00000001".to_string(),
                name: "Simple Echo Demo".to_string(),
                content: DefaultWorkflows::simple_echo(),
                is_default: true,
                is_edited: false,
            },
            WorkflowDefinition {
                id: "default-cursor-demo-00000002".to_string(),
                name: "Cursor Agent Demo".to_string(),
                content: DefaultWorkflows::cursor_demo(),
                is_default: true,
                is_edited: false,
            },
        ]
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AppSettings {
    pub theme: Theme,
    pub workflows: Vec<WorkflowDefinition>,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            theme: Theme::System,
            workflows: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WorkflowMetadata {
    pub id: String,
    pub workflow_name: String,
    pub start_timestamp: String,
    pub end_timestamp: Option<String>,
    pub status: WorkflowStatus,
    pub task_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskExecution {
    pub execution_id: String,
    pub task_id: String,
    pub task_name: String,
    pub status: crate::TaskStatus,
    pub logs: Vec<String>,
    pub start_timestamp: String,
    pub end_timestamp: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum WorkflowStatus {
    Running,
    Complete,
    Failed,
}

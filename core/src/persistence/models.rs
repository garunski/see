use crate::types::{AuditEntry, TaskInfo, TaskStatus};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// Workflow JSON structures with visualization metadata support
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NodePosition {
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct WorkflowVisualizationMetadata {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub node_positions: Option<HashMap<String, NodePosition>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WorkflowJson {
    pub id: String,
    pub name: String,
    pub tasks: Vec<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<WorkflowVisualizationMetadata>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WorkflowExecution {
    pub id: String,
    pub workflow_name: String,
    pub timestamp: String,
    pub success: bool,
    pub tasks: Vec<TaskInfo>,
    pub audit_trail: Vec<AuditEntry>,
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
    pub content: String,
    pub is_default: bool,
    pub is_edited: bool,
}

use crate::persistence::default_workflows::DefaultWorkflows;

impl WorkflowDefinition {
    pub fn get_name(&self) -> String {
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&self.content) {
            json.get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("Unnamed Workflow")
                .to_string()
        } else {
            "Invalid Workflow".to_string()
        }
    }

    pub fn get_default_workflows() -> Vec<Self> {
        vec![
            WorkflowDefinition {
                id: "default-echo-workflow-00000001".to_string(),
                content: DefaultWorkflows::simple_echo(),
                is_default: true,
                is_edited: false,
            },
            WorkflowDefinition {
                id: "default-cursor-demo-00000002".to_string(),
                content: DefaultWorkflows::cursor_demo(),
                is_default: true,
                is_edited: false,
            },
            WorkflowDefinition {
                id: "default-cursor-agent-simple-00000003".to_string(),
                content: DefaultWorkflows::cursor_agent_simple(),
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
    pub status: TaskStatus,
    pub logs: Vec<String>,
    pub start_timestamp: String,
    pub end_timestamp: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum WorkflowStatus {
    Running,
    Complete,
    Failed,
    WaitingForInput,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Prompt {
    pub id: String,          // human-readable ID like "generate-rust-code"
    pub content: String,     // the actual prompt text
    pub description: String, // optional description
    pub created_at: String,  // timestamp
}

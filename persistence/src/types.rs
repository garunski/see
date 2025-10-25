//! Additional types for GUI compatibility

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use tracing::debug;

/// Workflow definition for GUI compatibility
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WorkflowDefinition {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub content: String,
    pub is_default: bool,
    pub is_edited: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl WorkflowDefinition {
    /// Create a new workflow definition
    pub fn new(name: String, content: String) -> Self {
        debug!("Creating new workflow definition: {}", name);
        let now = Utc::now();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            description: None,
            content,
            is_default: false,
            is_edited: false,
            created_at: now,
            updated_at: now,
        }
    }
    
    /// Create a default workflow definition
    pub fn new_default(name: String, content: String) -> Self {
        debug!("Creating default workflow definition: {}", name);
        let now = Utc::now();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            description: None,
            content,
            is_default: true,
            is_edited: false,
            created_at: now,
            updated_at: now,
        }
    }
    
    /// Get the name of the workflow
    pub fn get_name(&self) -> &str {
        &self.name
    }
    
    /// Get default workflows
    pub fn get_default_workflows() -> Vec<Self> {
        debug!("Getting default workflows");
        vec![
            Self::new_default(
                "Simple Workflow".to_string(),
                r#"{
                    "name": "Simple Workflow",
                    "tasks": [
                        {
                            "id": "task1",
                            "name": "Task 1",
                            "type": "command",
                            "command": "echo 'Hello World'"
                        }
                    ]
                }"#.to_string()
            ),
            Self::new_default(
                "Complex Workflow".to_string(),
                r#"{
                    "name": "Complex Workflow",
                    "tasks": [
                        {
                            "id": "task1",
                            "name": "Setup",
                            "type": "command",
                            "command": "echo 'Setting up...'"
                        },
                        {
                            "id": "task2",
                            "name": "Process",
                            "type": "command",
                            "command": "echo 'Processing...'"
                        },
                        {
                            "id": "task3",
                            "name": "Cleanup",
                            "type": "command",
                            "command": "echo 'Cleaning up...'"
                        }
                    ]
                }"#.to_string()
            ),
        ]
    }
}

/// App settings for GUI compatibility
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AppSettings {
    pub theme: Theme,
    pub auto_save: bool,
    pub notifications: bool,
    pub default_workflow: Option<String>,
}

impl AppSettings {
    /// Create default app settings
    pub fn default() -> Self {
        debug!("Creating default app settings");
        Self {
            theme: Theme::System,
            auto_save: true,
            notifications: true,
            default_workflow: None,
        }
    }
}

/// Theme enum for GUI compatibility
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Theme {
    Light,
    Dark,
    System,
}

/// Workflow execution summary for GUI compatibility
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WorkflowExecutionSummary {
    pub id: String,
    pub workflow_name: String,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub success: Option<bool>,
    pub task_count: usize,
    pub timestamp: DateTime<Utc>,
}

/// Workflow metadata for GUI compatibility
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WorkflowMetadata {
    pub id: String,
    pub name: String,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub task_count: usize,
    pub workflow_name: String,
    pub start_timestamp: DateTime<Utc>,
    pub task_ids: Vec<String>,
}

/// Workflow JSON for GUI compatibility
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WorkflowJson {
    pub name: String,
    pub description: Option<String>,
    pub tasks: Vec<TaskInfo>,
}

/// Task info for GUI compatibility
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TaskInfo {
    pub id: String,
    pub name: String,
    pub status: String,
    pub created_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
}

/// Workflow status for GUI compatibility
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum WorkflowStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Paused,
}

impl WorkflowStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            WorkflowStatus::Pending => "pending",
            WorkflowStatus::Running => "running",
            WorkflowStatus::Completed => "completed",
            WorkflowStatus::Failed => "failed",
            WorkflowStatus::Paused => "paused",
        }
    }
}

/// Task status for GUI compatibility
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TaskStatus {
    Pending,
    InProgress,
    Complete,
    Failed,
    WaitingForInput,
}

impl TaskStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            TaskStatus::Pending => "pending",
            TaskStatus::InProgress => "in-progress",
            TaskStatus::Complete => "complete",
            TaskStatus::Failed => "failed",
            TaskStatus::WaitingForInput => "waiting-for-input",
        }
    }
}

/// Audit status for GUI compatibility
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AuditStatus {
    Success,
    Failure,
}

impl AuditStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            AuditStatus::Success => "success",
            AuditStatus::Failure => "failure",
        }
    }
}

impl std::fmt::Display for AuditStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Audit entry for GUI compatibility
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuditEntry {
    pub task_id: String,
    pub status: AuditStatus,
    pub timestamp: String,
    pub changes_count: usize,
    pub message: String,
}

/// Audit event for GUI compatibility
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuditEvent {
    pub id: String,
    pub workflow_id: String,
    pub task_id: Option<String>,
    pub event_type: String,
    pub message: String,
    pub timestamp: DateTime<Utc>,
    pub data: serde_json::Value,
}

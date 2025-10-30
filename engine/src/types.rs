use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EngineTask {
    pub id: String,
    pub name: String,
    pub function: TaskFunction,
    #[serde(default)]
    pub next_tasks: Vec<EngineTask>,
    #[serde(default)]
    pub status: TaskStatus,
    #[serde(default)]
    pub is_root: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "name", content = "input")]
pub enum TaskFunction {
    #[serde(rename = "cli_command")]
    CliCommand { command: String, args: Vec<String> },
    #[serde(rename = "cursor_agent")]
    CursorAgent { prompt: String, config: Value },
    #[serde(rename = "custom")]
    Custom { name: String, input: Value },
    #[serde(rename = "user_input")]
    UserInput {
        prompt: String,
        input_type: String,
        required: bool,
        default: Option<Value>,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EngineWorkflow {
    pub id: String,
    pub name: String,
    pub tasks: Vec<EngineTask>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TaskResult {
    pub success: bool,
    pub output: Value,
    pub error: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub enum TaskStatus {
    #[serde(rename = "pending")]
    #[default]
    Pending,
    #[serde(rename = "in_progress")]
    InProgress,
    #[serde(rename = "complete")]
    Complete,
    #[serde(rename = "failed")]
    Failed,
    #[serde(rename = "waiting_for_input")]
    WaitingForInput,
}

impl TaskStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            TaskStatus::Pending => "pending",
            TaskStatus::InProgress => "in_progress",
            TaskStatus::Complete => "complete",
            TaskStatus::Failed => "failed",
            TaskStatus::WaitingForInput => "waiting_for_input",
        }
    }
}

/// Result of workflow execution
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WorkflowResult {
    pub success: bool,
    pub workflow_name: String,
    pub tasks: Vec<TaskInfo>,
    pub audit_trail: Vec<AuditEntry>,
    pub per_task_logs: HashMap<String, Vec<String>>,
    pub errors: Vec<String>,
}

/// Task information for workflow results
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TaskInfo {
    pub id: String,
    pub name: String,
    pub status: TaskStatus,
}

/// Audit entry for workflow execution tracking
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AuditEntry {
    pub task_id: String,
    pub status: AuditStatus,
    pub timestamp: String,
    pub changes_count: usize,
    pub message: String,
}

/// Audit status for tracking task state changes
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AuditStatus {
    #[serde(rename = "success")]
    Success,
    #[serde(rename = "failure")]
    Failure,
}

impl std::fmt::Display for AuditStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AuditStatus::Success => write!(f, "Success"),
            AuditStatus::Failure => write!(f, "Failure"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ExecutionContext {
    pub execution_id: String,
    pub workflow_name: String,
    pub output_logs: Vec<String>,
    pub per_task_logs: HashMap<String, Vec<String>>,
    pub tasks: HashMap<String, EngineTask>,
}

impl ExecutionContext {
    pub fn new(execution_id: String, workflow_name: String) -> Self {
        Self {
            execution_id,
            workflow_name,
            output_logs: Vec::new(),
            per_task_logs: HashMap::new(),
            tasks: HashMap::new(),
        }
    }

    pub fn log(&mut self, message: String) {
        self.output_logs.push(message);
    }

    pub fn log_task(&mut self, task_id: String, message: String) {
        self.per_task_logs.entry(task_id).or_default().push(message);
    }

    pub fn update_task_status(&mut self, task_id: String, status: TaskStatus) {
        if let Some(task) = self.tasks.get_mut(&task_id) {
            task.status = status;
        }
    }
}

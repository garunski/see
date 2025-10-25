//! Core data types for the new workflow engine

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

/// Core task structure with recursive next_tasks support
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EngineTask {
    pub id: String,
    pub name: String,
    pub function: TaskFunction,
    pub next_tasks: Vec<EngineTask>,
    pub dependencies: Vec<String>, // For backward compatibility
    pub status: TaskStatus,
}

/// Task function types supported by the engine
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TaskFunction {
    #[serde(rename = "cli_command")]
    CliCommand {
        command: String,
        args: Vec<String>,
    },
    #[serde(rename = "cursor_agent")]
    CursorAgent {
        prompt: String,
        config: Value,
    },
    #[serde(rename = "custom")]
    Custom {
        name: String,
        input: Value,
    },
}

/// Workflow structure containing tasks
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EngineWorkflow {
    pub id: String,
    pub name: String,
    pub tasks: Vec<EngineTask>,
}

/// Result of task execution
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TaskResult {
    pub success: bool,
    pub output: Value,
    pub error: Option<String>,
}

/// Task execution status
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TaskStatus {
    #[serde(rename = "pending")]
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

impl Default for TaskStatus {
    fn default() -> Self {
        TaskStatus::Pending
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

/// Execution context for task handlers
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
        self.per_task_logs
            .entry(task_id)
            .or_insert_with(Vec::new)
            .push(message);
    }
    
    pub fn update_task_status(&mut self, task_id: String, status: TaskStatus) {
        if let Some(task) = self.tasks.get_mut(&task_id) {
            task.status = status;
        }
    }
}

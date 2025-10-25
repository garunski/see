//! Task execution model

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use tracing::debug;

/// Represents a task execution in the database
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskExecution {
    pub id: String,
    pub workflow_execution_id: String,  // Link to WorkflowExecution (NOT workflow_id!)
    pub task_id: String,                // Task ID from workflow definition
    pub name: String,
    pub status: String,                 // pending, running, completed, failed, waiting-for-input
    pub created_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub logs: Vec<String>,
    pub output: Option<serde_json::Value>,
    pub error: Option<String>,
    pub metadata: serde_json::Value,
}

impl TaskExecution {
    /// Create a new task execution
    pub fn new(workflow_execution_id: String, task_id: String, name: String) -> Self {
        debug!("Creating new task execution: {} for workflow execution {}", name, workflow_execution_id);
        Self {
            id: Uuid::new_v4().to_string(),
            workflow_execution_id,
            task_id,
            name,
            status: "pending".to_string(),
            created_at: Utc::now(),
            started_at: None,
            completed_at: None,
            logs: Vec::new(),
            output: None,
            error: None,
            metadata: serde_json::Value::Object(serde_json::Map::new()),
        }
    }
    
    /// Mark the task as started
    pub fn mark_started(&mut self) {
        debug!("Marking task {} as started", self.id);
        self.status = "running".to_string();
        self.started_at = Some(Utc::now());
    }
    
    /// Mark the task as completed
    pub fn mark_completed(&mut self, success: bool, output: Option<serde_json::Value>, error: Option<String>) {
        debug!("Marking task {} as completed (success: {})", self.id, success);
        self.status = if success { "completed".to_string() } else { "failed".to_string() };
        self.completed_at = Some(Utc::now());
        self.output = output;
        self.error = error;
    }
    
    /// Mark the task as waiting for input
    pub fn mark_waiting_for_input(&mut self) {
        debug!("Marking task {} as waiting for input", self.id);
        self.status = "waiting-for-input".to_string();
    }
    
    /// Resume a task that was waiting for input
    pub fn resume(&mut self) {
        debug!("Resuming task {}", self.id);
        self.status = "running".to_string();
    }
    
    /// Add a log message to the task
    pub fn add_log(&mut self, message: String) {
        debug!("Adding log to task {}: {}", self.id, message);
        self.logs.push(message);
    }
    
    /// Add multiple log messages
    pub fn add_logs(&mut self, messages: Vec<String>) {
        debug!("Adding {} logs to task {}", messages.len(), self.id);
        self.logs.extend(messages);
    }
    
    /// Clear all logs
    pub fn clear_logs(&mut self) {
        debug!("Clearing logs for task {}", self.id);
        self.logs.clear();
    }
    
    /// Get the task name
    pub fn get_name(&self) -> &str {
        &self.name
    }
}

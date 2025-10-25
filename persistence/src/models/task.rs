//! Task execution model

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use tracing::debug;

/// Represents a task execution in the database
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskExecution {
    pub id: String,
    pub workflow_id: String,
    pub task_name: String,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub success: Option<bool>,
    pub error_message: Option<String>,
    pub output: Option<String>,
    pub logs: Vec<String>,
    pub retry_count: i32,
    pub execution_order: i32,
    pub instance_id: Option<String>,
    pub last_updated: Option<DateTime<Utc>>,
}

impl TaskExecution {
    /// Create a new task execution
    pub fn new(workflow_id: String, task_name: String) -> Self {
        debug!("Creating new task execution: {} for workflow {}", task_name, workflow_id);
        Self {
            id: Uuid::new_v4().to_string(),
            workflow_id,
            task_name,
            status: "pending".to_string(),
            created_at: Utc::now(),
            started_at: None,
            completed_at: None,
            success: None,
            error_message: None,
            output: None,
            logs: Vec::new(),
            retry_count: 0,
            execution_order: 0,
            instance_id: None,
            last_updated: None,
        }
    }
    
    /// Mark the task as started
    pub fn mark_started(&mut self) {
        debug!("Marking task {} as started", self.id);
        self.status = "running".to_string();
        self.started_at = Some(Utc::now());
        self.last_updated = Some(Utc::now());
    }
    
    /// Mark the task as completed
    pub fn mark_completed(&mut self, success: bool, output: Option<String>, error_message: Option<String>) {
        debug!("Marking task {} as completed (success: {})", self.id, success);
        self.status = if success { "completed".to_string() } else { "failed".to_string() };
        self.completed_at = Some(Utc::now());
        self.success = Some(success);
        self.output = output;
        self.error_message = error_message;
        self.last_updated = Some(Utc::now());
    }
    
    /// Add a log message to the task
    pub fn add_log(&mut self, message: String) {
        debug!("Adding log to task {}: {}", self.id, message);
        self.logs.push(message);
        self.last_updated = Some(Utc::now());
    }
}

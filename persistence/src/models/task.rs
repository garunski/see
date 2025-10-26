//! TaskExecution model
//! 
//! This file contains ONLY TaskExecution struct following Single Responsibility Principle.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use crate::models::TaskStatus;

/// Individual task execution record
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TaskExecution {
    pub id: String,
    pub workflow_id: String,
    pub name: String,
    pub status: TaskStatus,
    pub output: Option<String>,
    pub error: Option<String>,
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

impl Default for TaskExecution {
    fn default() -> Self {
        let now = Utc::now();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            workflow_id: String::new(),
            name: String::new(),
            status: TaskStatus::Pending,
            output: None,
            error: None,
            created_at: now,
            completed_at: None,
        }
    }
}

impl TaskExecution {
    /// Validate task execution
    pub fn validate(&self) -> Result<(), String> {
        if self.id.is_empty() {
            return Err("Task ID cannot be empty".to_string());
        }
        if self.workflow_id.is_empty() {
            return Err("Workflow ID cannot be empty".to_string());
        }
        if self.name.is_empty() {
            return Err("Task name cannot be empty".to_string());
        }
        
        // Validate status consistency
        match self.status {
            TaskStatus::Complete | TaskStatus::Failed => {
                if self.completed_at.is_none() {
                    return Err("Completed tasks must have completion timestamp".to_string());
                }
            }
            TaskStatus::WaitingForInput => {
                if self.completed_at.is_some() {
                    return Err("Waiting tasks should not have completion timestamp".to_string());
                }
            }
            _ => {} // Pending, InProgress are fine
        }
        
        Ok(())
    }

    /// Check if task is finished (complete or failed)
    pub fn is_finished(&self) -> bool {
        matches!(self.status, TaskStatus::Complete | TaskStatus::Failed)
    }

    /// Check if task is waiting for input
    pub fn is_waiting_for_input(&self) -> bool {
        matches!(self.status, TaskStatus::WaitingForInput)
    }
}

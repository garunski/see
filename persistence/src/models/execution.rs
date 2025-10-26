//! Execution models
//! 
//! This file contains ONLY execution-related models following Single Responsibility Principle.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use crate::models::{WorkflowStatus, TaskExecution};

/// Full workflow execution record
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WorkflowExecution {
    pub id: String,
    pub workflow_name: String,
    pub status: WorkflowStatus,
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub success: bool,
    pub tasks: Vec<TaskExecution>,
    pub timestamp: DateTime<Utc>,
}

/// Lightweight execution summary
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WorkflowExecutionSummary {
    pub id: String,
    pub workflow_name: String,
    pub status: WorkflowStatus,
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub success: bool,
    pub task_count: usize,
    pub timestamp: DateTime<Utc>,
}

/// Basic workflow metadata
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WorkflowMetadata {
    pub id: String,
    pub name: String,
    pub status: String,  // "running" or other status
}

impl Default for WorkflowExecution {
    fn default() -> Self {
        let now = Utc::now();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            workflow_name: String::new(),
            status: WorkflowStatus::Pending,
            created_at: now,
            completed_at: None,
            success: false,
            tasks: Vec::new(),
            timestamp: now,
        }
    }
}

impl Default for WorkflowExecutionSummary {
    fn default() -> Self {
        let now = Utc::now();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            workflow_name: String::new(),
            status: WorkflowStatus::Pending,
            created_at: now,
            completed_at: None,
            success: false,
            task_count: 0,
            timestamp: now,
        }
    }
}

impl Default for WorkflowMetadata {
    fn default() -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name: String::new(),
            status: "pending".to_string(),
        }
    }
}

impl WorkflowExecution {
    /// Create a summary from this execution
    pub fn to_summary(&self) -> WorkflowExecutionSummary {
        WorkflowExecutionSummary {
            id: self.id.clone(),
            workflow_name: self.workflow_name.clone(),
            status: self.status.clone(),
            created_at: self.created_at,
            completed_at: self.completed_at,
            success: self.success,
            task_count: self.tasks.len(),
            timestamp: self.timestamp,
        }
    }

    /// Create metadata from this execution
    pub fn to_metadata(&self) -> WorkflowMetadata {
        WorkflowMetadata {
            id: self.id.clone(),
            name: self.workflow_name.clone(),
            status: self.status.to_string(),
        }
    }
}

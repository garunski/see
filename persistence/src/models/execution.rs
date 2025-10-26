//! Execution models
//!
//! This file contains ONLY execution-related models following Single Responsibility Principle.

use crate::models::{AuditEvent, TaskExecution, WorkflowStatus};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Full workflow execution record
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WorkflowExecution {
    pub id: String,
    pub workflow_name: String,
    pub workflow_snapshot: serde_json::Value,
    pub status: WorkflowStatus,
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub success: Option<bool>,
    pub tasks: Vec<TaskExecution>,
    pub timestamp: DateTime<Utc>,
    pub audit_trail: Vec<AuditEvent>,
    pub per_task_logs: HashMap<String, Vec<String>>,
    pub errors: Vec<String>,
}

/// Lightweight execution summary
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WorkflowExecutionSummary {
    pub id: String,
    pub workflow_name: String,
    pub status: WorkflowStatus,
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub success: Option<bool>,
    pub task_count: usize,
    pub timestamp: DateTime<Utc>,
}

/// Basic workflow metadata
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WorkflowMetadata {
    pub id: String,
    pub name: String,
    pub status: String,                 // "running" or other status
    pub workflow_name: String,          // For GUI compatibility
    pub start_timestamp: DateTime<Utc>, // For GUI compatibility
    pub task_ids: Vec<String>,          // For GUI compatibility
}

impl Default for WorkflowExecution {
    fn default() -> Self {
        let now = Utc::now();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            workflow_name: String::new(),
            workflow_snapshot: serde_json::json!({}),
            status: WorkflowStatus::Pending,
            created_at: now,
            completed_at: None,
            success: None,
            tasks: Vec::new(),
            timestamp: now,
            audit_trail: Vec::new(),
            per_task_logs: HashMap::new(),
            errors: Vec::new(),
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
            success: None,
            task_count: 0,
            timestamp: now,
        }
    }
}

impl Default for WorkflowMetadata {
    fn default() -> Self {
        let now = Utc::now();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name: String::new(),
            status: "pending".to_string(),
            workflow_name: String::new(),
            start_timestamp: now,
            task_ids: Vec::new(),
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
            workflow_name: self.workflow_name.clone(),
            start_timestamp: self.created_at,
            task_ids: self.tasks.iter().map(|t| t.id.clone()).collect(),
        }
    }
}

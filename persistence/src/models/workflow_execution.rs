//! Workflow execution model

use super::workflow::Workflow;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::debug;
use uuid::Uuid;

/// Represents a workflow execution in the database
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WorkflowExecution {
    pub id: String,
    pub workflow_id: String,         // Link to Workflow
    pub workflow_snapshot: Workflow, // Copy of workflow at execution time
    pub status: String,              // pending, running, completed, failed, paused
    pub created_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub success: Option<bool>,
    pub error_message: Option<String>,
    pub result: Option<serde_json::Value>,
    pub metadata: serde_json::Value,
    // GUI compatibility fields
    pub workflow_name: String,
    pub timestamp: DateTime<Utc>,
    pub tasks: Vec<crate::types::TaskInfo>,
    pub audit_trail: Vec<crate::types::AuditEntry>,
    pub per_task_logs: HashMap<String, Vec<String>>,
    pub errors: Vec<String>,
}

impl WorkflowExecution {
    /// Create a new workflow execution
    pub fn new(workflow: Workflow) -> Self {
        debug!(
            "Creating new workflow execution for workflow: {}",
            workflow.name
        );
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            workflow_id: workflow.id.clone(),
            workflow_snapshot: workflow.clone(),
            status: "pending".to_string(),
            created_at: now,
            started_at: None,
            completed_at: None,
            success: None,
            error_message: None,
            result: None,
            metadata: serde_json::Value::Object(serde_json::Map::new()),
            // GUI compatibility fields
            workflow_name: workflow.name.clone(),
            timestamp: now,
            tasks: Vec::new(),
            audit_trail: Vec::new(),
            per_task_logs: HashMap::new(),
            errors: Vec::new(),
        }
    }

    /// Mark the workflow as started
    pub fn mark_started(&mut self) {
        debug!("Marking workflow execution {} as started", self.id);
        self.status = "running".to_string();
        self.started_at = Some(Utc::now());
    }

    /// Mark the workflow as completed
    pub fn mark_completed(&mut self, success: bool, error_message: Option<String>) {
        debug!(
            "Marking workflow execution {} as completed (success: {})",
            self.id, success
        );
        self.status = if success {
            "completed".to_string()
        } else {
            "failed".to_string()
        };
        self.completed_at = Some(Utc::now());
        self.success = Some(success);
        self.error_message = error_message;
    }

    /// Mark the workflow as paused
    pub fn mark_paused(&mut self) {
        debug!("Marking workflow execution {} as paused", self.id);
        self.status = "paused".to_string();
    }

    /// Resume a paused workflow
    pub fn resume(&mut self) {
        debug!("Resuming workflow execution {}", self.id);
        self.status = "running".to_string();
    }

    /// Set the execution result
    pub fn set_result(&mut self, result: serde_json::Value) {
        debug!("Setting result for workflow execution {}", self.id);
        self.result = Some(result);
    }

    /// Get the workflow name from the snapshot
    pub fn get_workflow_name(&self) -> &str {
        &self.workflow_snapshot.name
    }
}

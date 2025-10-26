//! AuditEvent model
//!
//! This file contains ONLY AuditEvent struct following Single Responsibility Principle.

use crate::models::AuditStatus;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Audit trail entry
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AuditEvent {
    pub id: String,
    pub task_id: String,
    pub status: AuditStatus,
    pub timestamp: DateTime<Utc>,
    pub changes_count: usize,
    pub message: String,
}

impl Default for AuditEvent {
    fn default() -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            task_id: String::new(),
            status: AuditStatus::Success,
            timestamp: Utc::now(),
            changes_count: 0,
            message: String::new(),
        }
    }
}

impl AuditEvent {
    /// Validate audit event
    pub fn validate(&self) -> Result<(), String> {
        if self.id.is_empty() {
            return Err("Audit event ID cannot be empty".to_string());
        }
        if self.task_id.is_empty() {
            return Err("Task ID cannot be empty".to_string());
        }
        if self.message.is_empty() {
            return Err("Audit message cannot be empty".to_string());
        }

        Ok(())
    }

    /// Create a success audit event
    pub fn success(task_id: String, message: String, changes_count: usize) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            task_id,
            status: AuditStatus::Success,
            timestamp: Utc::now(),
            changes_count,
            message,
        }
    }

    /// Create a failure audit event
    pub fn failure(task_id: String, message: String, changes_count: usize) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            task_id,
            status: AuditStatus::Failure,
            timestamp: Utc::now(),
            changes_count,
            message,
        }
    }
}

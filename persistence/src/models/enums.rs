//! Enums for persistence layer
//!
//! This file contains ONLY enum definitions following Single Responsibility Principle.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Workflow execution status
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum WorkflowStatus {
    #[serde(rename = "pending")]
    Pending,
    #[serde(rename = "running")]
    Running,
    #[serde(rename = "complete")]
    Complete,
    #[serde(rename = "failed")]
    Failed,
}

impl fmt::Display for WorkflowStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WorkflowStatus::Pending => write!(f, "pending"),
            WorkflowStatus::Running => write!(f, "running"),
            WorkflowStatus::Complete => write!(f, "complete"),
            WorkflowStatus::Failed => write!(f, "failed"),
        }
    }
}

/// UI theme options
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Theme {
    #[serde(rename = "light")]
    Light,
    #[serde(rename = "dark")]
    Dark,
    #[serde(rename = "system")]
    System,
}

/// Task execution status (re-exported from engine)
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

impl TaskStatus {
    /// Convert TaskStatus to string representation
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

/// Audit entry status (re-exported from engine)
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

/// User input type
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum InputType {
    #[serde(rename = "string")]
    String,
    #[serde(rename = "number")]
    Number,
    #[serde(rename = "boolean")]
    Boolean,
}

impl std::fmt::Display for InputType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InputType::String => write!(f, "string"),
            InputType::Number => write!(f, "number"),
            InputType::Boolean => write!(f, "boolean"),
        }
    }
}

/// Input request status
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum InputRequestStatus {
    #[serde(rename = "pending")]
    Pending,
    #[serde(rename = "fulfilled")]
    Fulfilled,
}

impl std::fmt::Display for InputRequestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InputRequestStatus::Pending => write!(f, "pending"),
            InputRequestStatus::Fulfilled => write!(f, "fulfilled"),
        }
    }
}

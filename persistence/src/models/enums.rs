use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum WorkflowExecutionStatus {
    #[serde(rename = "pending")]
    Pending,
    #[serde(rename = "running")]
    Running,
    #[serde(rename = "complete")]
    Complete,
    #[serde(rename = "failed")]
    Failed,
    #[serde(rename = "waiting_for_input")]
    WaitingForInput,
}

impl WorkflowExecutionStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            WorkflowExecutionStatus::Pending => "pending",
            WorkflowExecutionStatus::Running => "running",
            WorkflowExecutionStatus::Complete => "complete",
            WorkflowExecutionStatus::Failed => "failed",
            WorkflowExecutionStatus::WaitingForInput => "waiting_for_input",
        }
    }
}

impl fmt::Display for WorkflowExecutionStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TaskExecutionStatus {
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

impl TaskExecutionStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            TaskExecutionStatus::Pending => "pending",
            TaskExecutionStatus::InProgress => "in_progress",
            TaskExecutionStatus::Complete => "complete",
            TaskExecutionStatus::Failed => "failed",
            TaskExecutionStatus::WaitingForInput => "waiting_for_input",
        }
    }
}

/// UI theme options
#[derive(Debug, Clone, PartialEq, Hash, Eq, Serialize, Deserialize)]
pub enum Theme {
    #[serde(rename = "light")]
    Light,
    #[serde(rename = "dark")]
    Dark,
    #[serde(rename = "system")]
    System,
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

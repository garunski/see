use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::Value;
use std::collections::HashMap;
use std::fmt;
use std::str::FromStr;

pub type OutputCallback = std::sync::Arc<dyn Fn(String) + Send + Sync>;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TaskStatus {
    Pending,
    InProgress,
    Complete,
    Failed,
    WaitingForInput,
}

impl TaskStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            TaskStatus::Pending => "pending",
            TaskStatus::InProgress => "in-progress",
            TaskStatus::Complete => "complete",
            TaskStatus::Failed => "failed",
            TaskStatus::WaitingForInput => "waiting-for-input",
        }
    }
}

impl fmt::Display for TaskStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl FromStr for TaskStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "pending" => Ok(TaskStatus::Pending),
            "in-progress" => Ok(TaskStatus::InProgress),
            "complete" => Ok(TaskStatus::Complete),
            "failed" => Ok(TaskStatus::Failed),
            "waiting-for-input" => Ok(TaskStatus::WaitingForInput),
            _ => Err(format!("Invalid task status: {}", s)),
        }
    }
}

impl Serialize for TaskStatus {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for TaskStatus {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        TaskStatus::from_str(&s).map_err(serde::de::Error::custom)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum AuditStatus {
    Success,
    Failure,
}

impl AuditStatus {
    pub fn as_http_code(&self) -> &'static str {
        match self {
            AuditStatus::Success => "200",
            AuditStatus::Failure => "500",
        }
    }

    pub fn from_http_code(code: &str) -> Self {
        match code {
            "200" => AuditStatus::Success,
            _ => AuditStatus::Failure,
        }
    }
}

impl fmt::Display for AuditStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_http_code())
    }
}

impl FromStr for AuditStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(AuditStatus::from_http_code(s))
    }
}

impl Serialize for AuditStatus {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.as_http_code())
    }
}

impl<'de> Deserialize<'de> for AuditStatus {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Ok(AuditStatus::from_http_code(&s))
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct TaskInfo {
    pub id: String,
    pub name: String,
    pub status: TaskStatus,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct AuditEntry {
    pub task_id: String,
    pub status: AuditStatus,
    pub timestamp: String,
    pub changes_count: usize,
    #[serde(default)]
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct WorkflowResult {
    pub success: bool,
    pub workflow_name: String,
    pub task_count: usize,
    pub execution_id: String,
    pub tasks: Vec<TaskInfo>,
    pub final_context: Value,
    pub audit_trail: Vec<AuditEntry>,
    pub per_task_logs: HashMap<String, Vec<String>>,
    pub errors: Vec<String>,
    pub output_logs: Vec<String>,
}

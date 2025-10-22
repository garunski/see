pub mod engine;
pub mod errors;
pub mod execution;
pub mod json_parser;
pub mod persistence;
pub mod task_executor;
pub mod utils;

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::Value;
use std::collections::HashMap;
use std::fmt;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::{Arc, Mutex, Once};

use crate::errors::CoreError;

/// Output callback type for real-time output during workflow execution
pub type OutputCallback = Arc<dyn Fn(String) + Send + Sync>;

/// Task execution status
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TaskStatus {
    Pending,
    InProgress,
    Complete,
    Failed,
}

impl TaskStatus {
    /// Convert to lowercase string for serialization
    pub fn as_str(&self) -> &'static str {
        match self {
            TaskStatus::Pending => "pending",
            TaskStatus::InProgress => "in-progress",
            TaskStatus::Complete => "complete",
            TaskStatus::Failed => "failed",
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

/// Audit trail status codes
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum AuditStatus {
    Success,
    Failure,
}

impl AuditStatus {
    /// Convert to HTTP status code string
    pub fn as_http_code(&self) -> &'static str {
        match self {
            AuditStatus::Success => "200",
            AuditStatus::Failure => "500",
        }
    }

    /// Create from HTTP status code
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

/// Information about a single task in a workflow
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct TaskInfo {
    pub id: String,
    pub name: String,
    pub status: TaskStatus,
}

/// Audit trail entry
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct AuditEntry {
    pub task_id: String,
    pub status: AuditStatus,
    pub timestamp: String,
    pub changes_count: usize,
}

/// Workflow execution result
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

/// Guard that MUST be kept alive for the duration of the program.
pub struct TracingGuard {
    _guard: tracing_appender::non_blocking::WorkerGuard,
}

/// Initialize tracing with console and file output.
pub fn init_tracing(log_dir: Option<PathBuf>) -> Result<TracingGuard, Box<CoreError>> {
    use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

    let log_dir = match log_dir {
        Some(dir) => dir,
        None => {
            let home = dirs::home_dir().ok_or_else(|| {
                Box::new(CoreError::Dataflow(
                    "Could not find home directory".to_string(),
                ))
            })?;
            home.join(".see").join("logs")
        }
    };

    std::fs::create_dir_all(&log_dir).map_err(|e| Box::new(CoreError::from(e)))?;

    let file_appender = tracing_appender::rolling::daily(log_dir, "see.log");
    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    tracing_subscriber::registry()
        .with(env_filter)
        .with(
            tracing_subscriber::fmt::layer()
                .with_target(true)
                .with_thread_ids(false)
                .with_line_number(true),
        )
        .with(
            tracing_subscriber::fmt::layer()
                .json()
                .with_writer(non_blocking),
        )
        .init();

    Ok(TracingGuard { _guard: guard })
}

/// Global store instance - singleton pattern to avoid multiple database connections
static GLOBAL_STORE: Mutex<Option<Arc<dyn AuditStore + Send + Sync>>> = Mutex::new(None);
static INIT: Once = Once::new();

/// Get the global store instance, creating it if it doesn't exist
pub fn get_global_store() -> Result<Arc<dyn AuditStore + Send + Sync>, Box<CoreError>> {
    INIT.call_once(|| {
        let store = match RedbStore::new_default() {
            Ok(store) => Some(Arc::new(store) as Arc<dyn AuditStore + Send + Sync>),
            Err(e) => {
                eprintln!("Failed to initialize global store: {}", e);
                None
            }
        };
        if let Ok(mut global_store) = GLOBAL_STORE.lock() {
            *global_store = store;
        }
    });

    let global_store = GLOBAL_STORE.lock().map_err(|e| {
        Box::new(CoreError::MutexLock(format!(
            "Failed to lock global store: {}",
            e
        )))
    })?;
    global_store
        .as_ref()
        .ok_or_else(|| {
            Box::new(CoreError::Dataflow(
                "Global store not initialized".to_string(),
            ))
        })
        .cloned()
}

pub use crate::engine::execute::{execute_workflow, execute_workflow_from_content};
pub use crate::persistence::models::{
    AppSettings, TaskExecution, Theme, WorkflowDefinition, WorkflowExecution,
    WorkflowExecutionSummary, WorkflowMetadata, WorkflowStatus,
};
pub use crate::persistence::redb_store::{AuditStore, RedbStore};

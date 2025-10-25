#![allow(clippy::result_large_err)]
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CoreError {
    #[error("dataflow error: {0}")]
    Dataflow(String),

    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("execution context error: {0}")]
    ExecutionContext(String),

    #[error("mutex lock error: {0}")]
    MutexLock(String),

    #[error("workflow execution error: {0}")]
    WorkflowExecution(String),

    #[error("task execution error: {0}")]
    TaskExecution(String),

    #[error("command execution error: {0}")]
    CommandExecution(String),

    #[error("validation error: {0}")]
    Validation(String),

    #[error("serialization error: {0}")]
    Serialization(String),


    #[error("not found: {resource} with ID '{id}'")]
    NotFound { resource: String, id: String },
}

impl From<Box<CoreError>> for CoreError {
    fn from(err: Box<CoreError>) -> Self {
        *err
    }
}

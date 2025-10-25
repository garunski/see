//! Error types for the new workflow engine

use thiserror::Error;

#[derive(Error, Debug)]
pub enum EngineError {
    #[error("Parser error: {0}")]
    Parser(#[from] ParserError),

    #[error("Handler error: {0}")]
    Handler(#[from] HandlerError),

    #[error("Execution error: {0}")]
    Execution(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

#[derive(Error, Debug)]
pub enum ParserError {
    #[error("JSON parse error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Missing required field: {0}")]
    MissingField(String),

    #[error("Invalid task structure: {0}")]
    InvalidTask(String),
}

#[derive(Error, Debug)]
pub enum HandlerError {
    #[error("Task execution failed: {0}")]
    ExecutionFailed(String),

    #[error("Handler not found: {0}")]
    HandlerNotFound(String),

    #[error("Invalid task configuration: {0}")]
    InvalidConfiguration(String),
}

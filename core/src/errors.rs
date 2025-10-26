// Core error types ONLY

use std::{fmt, option, convert};
use thiserror::Error;

/// Main error type for core crate operations
#[derive(Error, Debug)]
pub enum CoreError {
    #[error("Engine error: {0}")]
    Engine(#[from] engine::EngineError),
    
    #[error("Persistence error: {0}")]
    Persistence(String),
    
    #[error("Workflow not found: {0}")]
    WorkflowNotFound(String),
    
    #[error("Task not found: {0}")]
    TaskNotFound(String),
    
    #[error("Execution error: {0}")]
    Execution(String),
}

impl From<String> for CoreError {
    fn from(err: String) -> Self {
        CoreError::Persistence(err)
    }
}

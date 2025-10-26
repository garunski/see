// Core error types ONLY

/// Main error type for core crate operations
#[derive(thiserror::Error, Debug)]
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

    #[error("Invalid input type: {0}")]
    InvalidInputType(String),

    #[error("Input value required but not provided")]
    InputRequired,

    #[error("Input validation failed: {0}")]
    InputValidationFailed(String),

    #[error("Task is not waiting for input")]
    TaskNotWaitingForInput,

    #[error("Workflow is waiting for user input")]
    WorkflowWaitingForInput,
}

impl From<String> for CoreError {
    fn from(err: String) -> Self {
        CoreError::Persistence(err)
    }
}

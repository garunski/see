// Bridge module - Type conversions between engine and persistence

pub mod audit;
pub mod execution;
pub mod task;
pub mod user_input;
pub mod workflow;

// Re-export bridge types
pub use workflow::OutputCallback;
pub use workflow::WorkflowResult;

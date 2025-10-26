// Bridge module - Type conversions between engine and persistence

pub mod workflow;
pub mod execution;
pub mod task;
pub mod audit;

// Re-export bridge types
pub use workflow::WorkflowResult;
pub use workflow::OutputCallback;

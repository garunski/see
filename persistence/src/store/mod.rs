//! Store modules for CRUD operations

pub mod workflows;
pub mod workflow_executions;
pub mod task_executions;
pub mod user_prompts;
pub mod ai_prompts;
pub mod settings;

// Re-export all stores
pub use workflows::WorkflowStore;
pub use workflow_executions::WorkflowExecutionStore;
pub use task_executions::TaskExecutionStore;
pub use user_prompts::UserPromptStore;
pub use ai_prompts::AiPromptStore;
pub use settings::SettingsStore;

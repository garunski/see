//! Store modules for CRUD operations

pub mod ai_prompts;
pub mod settings;
pub mod task_executions;
pub mod user_prompts;
pub mod workflow_executions;
pub mod workflows;

// Re-export all stores
pub use ai_prompts::AiPromptStore;
pub use settings::SettingsStore;
pub use task_executions::TaskExecutionStore;
pub use user_prompts::UserPromptStore;
pub use workflow_executions::WorkflowExecutionStore;
pub use workflows::WorkflowStore;

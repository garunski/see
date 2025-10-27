// API module - Core API functions

pub mod defaults;
pub mod execution;
pub mod init;
pub mod input;
pub mod resume;
pub mod system_templates;

// Re-export API functions
pub use defaults::get_default_workflows;
pub use execution::execute_workflow_by_id;
pub use init::{init_global_store, init_tracing};
pub use input::{get_pending_inputs, get_tasks_waiting_for_input, provide_user_input};
pub use resume::resume_task;
pub use system_templates::{
    clone_system_prompt, clone_system_workflow, load_all_system_templates, load_system_prompts,
    load_system_workflows,
};

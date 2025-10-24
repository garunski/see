/// Workflow editor module
///
/// This module contains the refactored workflow editor components,
/// extracted from the original 662-line monolith into focused,
/// single-responsibility modules.
mod handlers;
mod javascript_templates;
mod page;

pub use handlers::{
    create_reset_to_default_handler, create_save_workflow_handler, create_switch_to_json_handler,
    create_switch_to_visual_handler, SaveWorkflowParams,
};
pub use javascript_templates::{load_workflow_script, MESSAGE_LISTENER_SCRIPT};

// Re-export the main components
pub use page::{EditMode, WorkflowEditPage, WorkflowEditPageNew};

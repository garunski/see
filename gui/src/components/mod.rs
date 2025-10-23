pub mod sidebar;
pub mod step_navigator;
pub mod ui;
pub mod workflow_progress;

pub use sidebar::Sidebar;
pub use step_navigator::StepNavigator;
pub use ui::{Button, ButtonSize, ButtonVariant, ConfirmDialog};
pub use workflow_progress::WorkflowProgress;

#[derive(Debug, Clone, PartialEq)]
pub enum ExecutionStatus {
    Idle,
    Running,
    Complete,
    Failed,
}

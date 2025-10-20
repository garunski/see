pub mod context_panel;
pub mod errors_panel;
pub mod output_logs_panel;
pub mod sidebar;
pub mod step_navigator;
pub mod toast;
pub mod workflow_info_card;
pub mod workflow_progress;

pub use context_panel::ContextPanel;
pub use errors_panel::ErrorsPanel;
pub use output_logs_panel::OutputLogsPanel;
pub use sidebar::Sidebar;
pub use step_navigator::StepNavigator;
pub use toast::Toast;
pub use workflow_info_card::WorkflowInfoCard;
pub use workflow_progress::WorkflowProgress;

#[derive(Debug, Clone, PartialEq)]
pub enum ExecutionStatus {
    Idle,
    Running,
    Complete,
    Failed,
}

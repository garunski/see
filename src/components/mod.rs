pub mod context_panel;
pub mod errors_panel;
pub mod output_logs_panel;
pub mod sidebar;
pub mod toast;
pub mod workflow_info_card;

pub use context_panel::ContextPanel;
pub use errors_panel::ErrorsPanel;
pub use output_logs_panel::OutputLogsPanel;
pub use sidebar::Sidebar;
pub use toast::Toast;
pub use workflow_info_card::WorkflowInfoCard;

// Move ExecutionStatus enum to components module
#[derive(Debug, Clone, PartialEq)]
pub enum ExecutionStatus {
    Idle,
    Running,
    Complete,
    Failed,
}

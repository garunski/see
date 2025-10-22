pub mod sidebar;
pub mod status_bar;

pub use sidebar::Sidebar;
pub use status_bar::*;

#[derive(Debug, Clone, PartialEq)]
pub enum ExecutionStatus {
    Idle,
    Running,
    Complete,
    Failed,
}

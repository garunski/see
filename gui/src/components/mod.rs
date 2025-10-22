pub mod sidebar;

pub use sidebar::Sidebar;

#[derive(Debug, Clone, PartialEq)]
pub enum ExecutionStatus {
    Idle,
    Running,
    Complete,
    Failed,
}

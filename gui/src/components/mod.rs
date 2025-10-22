pub mod sidebar;
pub mod ui;

pub use sidebar::Sidebar;
// UI components are available through individual imports as needed

#[derive(Debug, Clone, PartialEq)]
pub enum ExecutionStatus {
    Idle,
    Running,
    Complete,
    Failed,
}

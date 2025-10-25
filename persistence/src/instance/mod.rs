//! Multi-instance management and coordination

pub mod manager;
pub mod coordinator;

// Re-export public types
pub use manager::InstanceManager;
pub use coordinator::{MultiInstanceCoordinator, InstanceStats};

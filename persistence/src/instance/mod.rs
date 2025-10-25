//! Multi-instance management and coordination

pub mod coordinator;
pub mod manager;

// Re-export public types
pub use coordinator::{InstanceStats, MultiInstanceCoordinator};
pub use manager::InstanceManager;

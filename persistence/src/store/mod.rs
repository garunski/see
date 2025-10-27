//! Store operations for persistence layer
//!
//! Each store operation is in its own file following Single Responsibility Principle.

pub mod audit;
pub mod execution;
pub mod lib;
pub mod prompt;
pub mod settings;
pub mod task;
pub mod user_input;
pub mod utils;
pub mod workflow;

// Re-export Store struct
pub use lib::Store;

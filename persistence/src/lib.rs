//! Persistence layer for S.E.E. workflow engine
//! 
//! This crate provides SQLite-based persistence using sqlx for multi-process concurrent access.
//! It follows the Single Responsibility Principle (SRP) with each file having one clear responsibility.

pub mod models;
pub mod store;
pub mod errors;
pub mod logging;

// Re-export all public types for convenience
pub use models::*;
pub use store::Store;
pub use errors::PersistenceError;

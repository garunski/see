//! PersistenceError definitions
//!
//! This file contains ONLY error types following Single Responsibility Principle.

use thiserror::Error;

/// Main error type for persistence operations
#[derive(Error, Debug)]
pub enum PersistenceError {
    #[error("Database error: {0}")]
    Database(String),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Transaction error: {0}")]
    Transaction(String),

    #[error("Connection pool error: {0}")]
    ConnectionPool(String),
}

impl From<sqlx::Error> for PersistenceError {
    fn from(err: sqlx::Error) -> Self {
        PersistenceError::Database(err.to_string())
    }
}

impl From<String> for PersistenceError {
    fn from(err: String) -> Self {
        PersistenceError::Database(err)
    }
}

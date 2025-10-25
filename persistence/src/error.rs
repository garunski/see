//! Error types for the persistence layer

use thiserror::Error;

/// Errors that can occur in the persistence layer
#[derive(Error, Debug)]
pub enum PersistenceError {
    #[error("Database connection error: {0}")]
    Connection(String),
    
    #[error("SQL execution error: {0}")]
    Sql(#[from] rusqlite::Error),
    
    #[error("Serialization error: {0}")]
    Serialization(String),
    
    #[error("Pool error: {0}")]
    Pool(#[from] r2d2::Error),
    
    #[error("Validation error: {message} (field: {field})")]
    Validation { message: String, field: String },
    
    #[error("Not found: {resource} with ID '{id}'")]
    NotFound { resource: String, id: String },
    
    #[error("Concurrency error: {0}")]
    Concurrency(String),
    
    #[error("Instance error: {0}")]
    Instance(String),
}

// Remove this implementation since it conflicts with the derive macro

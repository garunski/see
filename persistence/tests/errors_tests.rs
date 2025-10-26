//! Tests for PersistenceError
//! 
//! Tests error variants, error conversions following Single Responsibility Principle.

use persistence::PersistenceError;

#[test]
fn test_persistence_error_database() {
    let error = PersistenceError::Database("Connection failed".to_string());
    
    assert!(matches!(error, PersistenceError::Database(_)));
    
    // Test Display implementation
    let error_string = format!("{}", error);
    assert!(error_string.contains("Database error"));
    assert!(error_string.contains("Connection failed"));
}

#[test]
fn test_persistence_error_serialization() {
    // Create a real serde_json::Error
    let json_error = serde_json::from_str::<serde_json::Value>("invalid json").unwrap_err();
    let error = PersistenceError::Serialization(json_error);
    
    assert!(matches!(error, PersistenceError::Serialization(_)));
    
    // Test Display implementation
    let error_string = format!("{}", error);
    assert!(error_string.contains("Serialization error"));
}

#[test]
fn test_persistence_error_io() {
    // Create a real std::io::Error
    let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "File not found");
    let error = PersistenceError::Io(io_error);
    
    assert!(matches!(error, PersistenceError::Io(_)));
    
    // Test Display implementation
    let error_string = format!("{}", error);
    assert!(error_string.contains("IO error"));
}

#[test]
fn test_persistence_error_transaction() {
    let error = PersistenceError::Transaction("Transaction failed".to_string());
    
    assert!(matches!(error, PersistenceError::Transaction(_)));
    
    // Test Display implementation
    let error_string = format!("{}", error);
    assert!(error_string.contains("Transaction error"));
    assert!(error_string.contains("Transaction failed"));
}

#[test]
fn test_persistence_error_from_sqlx_error() {
    // Test conversion from sqlx::Error
    let sqlx_error = sqlx::Error::PoolClosed;
    let persistence_error: PersistenceError = sqlx_error.into();
    
    assert!(matches!(persistence_error, PersistenceError::Database(_)));
    
    // Test Display implementation
    let error_string = format!("{}", persistence_error);
    assert!(error_string.contains("Database error"));
}

#[test]
fn test_persistence_error_from_serde_json_error() {
    // Test conversion from serde_json::Error
    let json_error = serde_json::from_str::<serde_json::Value>("invalid json").unwrap_err();
    let persistence_error: PersistenceError = json_error.into();
    
    assert!(matches!(persistence_error, PersistenceError::Serialization(_)));
    
    // Test Display implementation
    let error_string = format!("{}", persistence_error);
    assert!(error_string.contains("Serialization error"));
}

#[test]
fn test_persistence_error_from_io_error() {
    // Test conversion from std::io::Error
    let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "File not found");
    let persistence_error: PersistenceError = io_error.into();
    
    assert!(matches!(persistence_error, PersistenceError::Io(_)));
    
    // Test Display implementation
    let error_string = format!("{}", persistence_error);
    assert!(error_string.contains("IO error"));
}

#[test]
fn test_persistence_error_debug() {
    let error = PersistenceError::Database("Test error".to_string());
    
    // Test Debug implementation
    let debug_string = format!("{:?}", error);
    assert!(debug_string.contains("Database"));
    assert!(debug_string.contains("Test error"));
}

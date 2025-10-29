//! Tests for logging functionality
//!
//! Tests log level configuration, structured logging following Single Responsibility Principle.

use s_e_e_persistence::logging::{
    init_logging, log_db_operation_error, log_db_operation_start, log_db_operation_success,
    log_deserialization, log_serialization,
};
use std::sync::Once;

static INIT: Once = Once::new();

fn init_test_logging() {
    INIT.call_once(|| {
        // Initialize logging for tests
        let _ = init_logging(None);
    });
}

#[test]
fn test_log_db_operation_start() {
    init_test_logging();

    // This should not panic
    log_db_operation_start("test_operation", "test_table");
}

#[test]
fn test_log_db_operation_success() {
    init_test_logging();

    // This should not panic
    log_db_operation_success("test_operation", "test_table", 100);
}

#[test]
fn test_log_db_operation_error() {
    init_test_logging();

    // This should not panic
    log_db_operation_error("test_operation", "test_table", "Test error message");
}

#[test]
fn test_log_serialization() {
    init_test_logging();

    // This should not panic
    log_serialization("test_object", 1024);
}

#[test]
fn test_log_deserialization() {
    init_test_logging();

    // This should not panic
    log_deserialization("test_object", 2048);
}

#[test]
fn test_logging_multiple_operations() {
    init_test_logging();

    // Test multiple logging operations
    for i in 0..10 {
        log_db_operation_start(&format!("operation_{}", i), "test_table");
        log_db_operation_success(&format!("operation_{}", i), "test_table", i * 10);
        log_serialization(&format!("object_{}", i), (i * 100) as usize);
    }
}

#[test]
fn test_logging_error_scenarios() {
    init_test_logging();

    // Test various error scenarios
    let error_messages = vec![
        "Connection timeout",
        "Invalid SQL syntax",
        "Constraint violation",
        "Deadlock detected",
        "Permission denied",
    ];

    for error_msg in error_messages {
        log_db_operation_error("error_test", "test_table", error_msg);
    }
}

#[test]
fn test_logging_large_data() {
    init_test_logging();

    // Test logging with large data sizes
    let large_sizes = vec![1024, 10240, 102400, 1048576]; // 1KB, 10KB, 100KB, 1MB

    for size in large_sizes {
        log_serialization("large_object", size);
        log_deserialization("large_object", size);
    }
}

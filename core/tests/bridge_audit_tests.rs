// Audit conversion tests ONLY

use chrono::Datelike;
use s_e_e_engine::{AuditEntry, AuditStatus as EngineAuditStatus};
use s_e_e_persistence::AuditStatus as PersistenceAuditStatus;
use s_e_e_core::{bridge::*, CoreError};

#[test]
fn test_audit_status_conversion() {
    // Test all engine to persistence audit status conversions
    let test_cases = vec![
        (EngineAuditStatus::Success, PersistenceAuditStatus::Success),
        (EngineAuditStatus::Failure, PersistenceAuditStatus::Failure),
    ];

    for (engine_status, expected_persistence_status) in test_cases {
        let audit_entry = AuditEntry {
            task_id: "task-1".to_string(),
            status: engine_status,
            timestamp: "2024-01-15T10:30:45Z".to_string(),
            changes_count: 0,
            message: "Test".to_string(),
        };

        let audit_event = audit::audit_entry_to_event(&audit_entry).unwrap();
        assert_eq!(audit_event.status, expected_persistence_status);
    }
}

#[test]
fn test_timestamp_parsing() {
    let valid_timestamps = vec![
        "2024-01-15T10:30:45Z",
        "2024-01-15T10:30:45.123Z",
        "2024-01-15T10:30:45+00:00",
        "2024-01-15T10:30:45.123456789Z",
    ];

    for timestamp_str in valid_timestamps {
        let audit_entry = AuditEntry {
            task_id: "task-1".to_string(),
            status: EngineAuditStatus::Success,
            timestamp: timestamp_str.to_string(),
            changes_count: 0,
            message: "Test".to_string(),
        };

        let result = audit::audit_entry_to_event(&audit_entry);
        assert!(
            result.is_ok(),
            "Failed to parse timestamp: {}",
            timestamp_str
        );

        let audit_event = result.unwrap();
        assert_eq!(audit_event.timestamp.year(), 2024);
        assert_eq!(audit_event.timestamp.month(), 1);
        assert_eq!(audit_event.timestamp.day(), 15);
    }
}

#[test]
fn test_invalid_timestamp_parsing() {
    let invalid_timestamps = vec![
        "invalid timestamp",
        "2024-13-45T25:70:90Z", // Invalid date/time
        "not-a-timestamp",
        "",
    ];

    for timestamp_str in invalid_timestamps {
        let audit_entry = AuditEntry {
            task_id: "task-1".to_string(),
            status: EngineAuditStatus::Success,
            timestamp: timestamp_str.to_string(),
            changes_count: 0,
            message: "Test".to_string(),
        };

        let result = audit::audit_entry_to_event(&audit_entry);
        assert!(
            result.is_err(),
            "Should fail for invalid timestamp: {}",
            timestamp_str
        );

        match result.unwrap_err() {
            CoreError::Execution(msg) => {
                assert!(msg.contains("Invalid timestamp"));
            }
            other => panic!("Expected Execution error, got: {:?}", other),
        }
    }
}

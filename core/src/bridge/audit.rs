// Audit conversions ONLY

use engine::AuditEntry;
use persistence::{AuditEvent, AuditStatus as PersistenceAuditStatus};
use engine::AuditStatus as EngineAuditStatus;
use crate::errors::CoreError;

/// Convert AuditEntry to AuditEvent
pub fn audit_entry_to_event(
    entry: &AuditEntry,
) -> Result<AuditEvent, CoreError> {
    // Parse RFC3339 timestamp
    let timestamp = chrono::DateTime::parse_from_rfc3339(&entry.timestamp)
        .map_err(|e| CoreError::Execution(format!("Invalid timestamp: {}", e)))?
        .with_timezone(&chrono::Utc);
    
    // Convert engine AuditStatus to persistence AuditStatus
    let persistence_status = match entry.status {
        EngineAuditStatus::Success => PersistenceAuditStatus::Success,
        EngineAuditStatus::Failure => PersistenceAuditStatus::Failure,
    };
    
    Ok(AuditEvent {
        id: uuid::Uuid::new_v4().to_string(),
        task_id: entry.task_id.clone(),
        status: persistence_status,
        timestamp,
        changes_count: entry.changes_count,
        message: entry.message.clone(),
    })
}

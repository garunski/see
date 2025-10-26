//! Audit store operations
//!
//! This file contains ONLY audit operations following Single Responsibility Principle.

use super::Store;
use crate::logging::{
    log_db_operation_error, log_db_operation_start, log_db_operation_success, log_serialization,
};
use crate::models::AuditEvent;

impl Store {
    /// Log an audit event
    pub async fn log_audit_event(&self, event: AuditEvent) -> Result<(), String> {
        log_db_operation_start("log_audit_event", "audit_events");

        let json_data = serde_json::to_string(&event).map_err(|e| {
            log_db_operation_error("log_audit_event", "audit_events", &e.to_string());
            format!("Serialization error: {}", e)
        })?;

        log_serialization("AuditEvent", json_data.len());

        sqlx::query("INSERT OR REPLACE INTO audit_events (id, data) VALUES (?, ?)")
            .bind(&event.id)
            .bind(&json_data)
            .execute(self.pool())
            .await
            .map_err(|e| {
                log_db_operation_error("log_audit_event", "audit_events", &e.to_string());
                format!("Database error: {}", e)
            })?;

        log_db_operation_success("log_audit_event", "audit_events", 0);
        Ok(())
    }
}

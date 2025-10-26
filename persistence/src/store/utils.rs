//! Utility store operations
//! 
//! This file contains ONLY utility functions following Single Responsibility Principle.

use crate::logging::{log_db_operation_start, log_db_operation_success, log_db_operation_error};
use super::Store;

impl Store {
    /// Clear all data from all tables
    pub async fn clear_all_data(&self) -> Result<(), String> {
        log_db_operation_start("clear_all_data", "all");
        
        let tables = [
            "workflows",
            "workflow_executions", 
            "task_executions",
            "user_prompts",
            "audit_events",
            "settings"
        ];

        for table in &tables {
            sqlx::query(&format!("DELETE FROM {}", table))
                .execute(self.pool())
                .await
                .map_err(|e| {
                    log_db_operation_error("clear_all_data", table, &e.to_string());
                    format!("Database error: {}", e)
                })?;
        }
        
        log_db_operation_success("clear_all_data", "all", 0);
        Ok(())
    }
}

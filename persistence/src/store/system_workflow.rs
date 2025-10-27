//! System workflow store operations
//!
//! This file contains ONLY system workflow CRUD operations following Single Responsibility Principle.

use super::Store;
use crate::logging::{
    log_db_operation_error, log_db_operation_start, log_db_operation_success, log_deserialization,
    log_serialization,
};
use crate::models::SystemWorkflow;
use sqlx::Row;

impl Store {
    /// Save a system workflow definition
    pub async fn save_system_workflow(&self, workflow: &SystemWorkflow) -> Result<(), String> {
        log_db_operation_start("save_system_workflow", "system_workflows");

        let json_data = serde_json::to_string(workflow).map_err(|e| {
            log_db_operation_error("save_system_workflow", "system_workflows", &e.to_string());
            format!("Serialization error: {}", e)
        })?;

        log_serialization("SystemWorkflow", json_data.len());

        sqlx::query("INSERT OR REPLACE INTO system_workflows (id, data) VALUES (?, ?)")
            .bind(&workflow.id)
            .bind(&json_data)
            .execute(self.pool())
            .await
            .map_err(|e| {
                log_db_operation_error("save_system_workflow", "system_workflows", &e.to_string());
                format!("Database error: {}", e)
            })?;

        log_db_operation_success("save_system_workflow", "system_workflows", 0);
        Ok(())
    }

    /// Get a system workflow definition by ID
    pub async fn get_system_workflow(&self, id: &str) -> Result<Option<SystemWorkflow>, String> {
        log_db_operation_start("get_system_workflow", "system_workflows");

        let row = sqlx::query("SELECT data FROM system_workflows WHERE id = ?")
            .bind(id)
            .fetch_optional(self.pool())
            .await
            .map_err(|e| {
                log_db_operation_error("get_system_workflow", "system_workflows", &e.to_string());
                format!("Database error: {}", e)
            })?;

        match row {
            Some(row) => {
                let json_data: String = row.get("data");
                log_deserialization("SystemWorkflow", json_data.len());

                let workflow = serde_json::from_str(&json_data).map_err(|e| {
                    log_db_operation_error(
                        "get_system_workflow",
                        "system_workflows",
                        &e.to_string(),
                    );
                    format!("Deserialization error: {}", e)
                })?;
                Ok(Some(workflow))
            }
            None => Ok(None),
        }
    }

    /// List all system workflow definitions
    pub async fn list_system_workflows(&self) -> Result<Vec<SystemWorkflow>, String> {
        log_db_operation_start("list_system_workflows", "system_workflows");

        let rows = sqlx::query("SELECT data FROM system_workflows ORDER BY id")
            .fetch_all(self.pool())
            .await
            .map_err(|e| {
                log_db_operation_error("list_system_workflows", "system_workflows", &e.to_string());
                format!("Database error: {}", e)
            })?;

        let mut workflows = Vec::new();
        for row in rows {
            let json_data: String = row.get("data");
            log_deserialization("SystemWorkflow", json_data.len());

            let workflow = serde_json::from_str(&json_data).map_err(|e| {
                log_db_operation_error("list_system_workflows", "system_workflows", &e.to_string());
                format!("Deserialization error: {}", e)
            })?;
            workflows.push(workflow);
        }

        log_db_operation_success(
            "list_system_workflows",
            "system_workflows",
            workflows.len() as u64,
        );
        Ok(workflows)
    }

    /// Delete a system workflow definition
    pub async fn delete_system_workflow(&self, id: &str) -> Result<(), String> {
        log_db_operation_start("delete_system_workflow", "system_workflows");

        sqlx::query("DELETE FROM system_workflows WHERE id = ?")
            .bind(id)
            .execute(self.pool())
            .await
            .map_err(|e| {
                log_db_operation_error(
                    "delete_system_workflow",
                    "system_workflows",
                    &e.to_string(),
                );
                format!("Database error: {}", e)
            })?;

        log_db_operation_success("delete_system_workflow", "system_workflows", 0);
        Ok(())
    }

    /// Clear all system workflows (useful for reloading)
    pub async fn clear_system_workflows(&self) -> Result<(), String> {
        log_db_operation_start("clear_system_workflows", "system_workflows");

        sqlx::query("DELETE FROM system_workflows")
            .execute(self.pool())
            .await
            .map_err(|e| {
                log_db_operation_error(
                    "clear_system_workflows",
                    "system_workflows",
                    &e.to_string(),
                );
                format!("Database error: {}", e)
            })?;

        log_db_operation_success("clear_system_workflows", "system_workflows", 0);
        Ok(())
    }
}

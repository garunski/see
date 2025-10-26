//! Workflow store operations
//!
//! This file contains ONLY workflow CRUD operations following Single Responsibility Principle.

use super::Store;
use crate::logging::{
    log_db_operation_error, log_db_operation_start, log_db_operation_success, log_deserialization,
    log_serialization,
};
use crate::models::WorkflowDefinition;
use sqlx::Row;

impl Store {
    /// Save a workflow definition
    pub async fn save_workflow(&self, workflow: &WorkflowDefinition) -> Result<(), String> {
        log_db_operation_start("save_workflow", "workflows");

        let json_data = serde_json::to_string(workflow).map_err(|e| {
            log_db_operation_error("save_workflow", "workflows", &e.to_string());
            format!("Serialization error: {}", e)
        })?;

        log_serialization("WorkflowDefinition", json_data.len());

        sqlx::query("INSERT OR REPLACE INTO workflows (id, data) VALUES (?, ?)")
            .bind(&workflow.id)
            .bind(&json_data)
            .execute(self.pool())
            .await
            .map_err(|e| {
                log_db_operation_error("save_workflow", "workflows", &e.to_string());
                format!("Database error: {}", e)
            })?;

        log_db_operation_success("save_workflow", "workflows", 0);
        Ok(())
    }

    /// Get a workflow definition by ID
    pub async fn get_workflow(&self, id: &str) -> Result<Option<WorkflowDefinition>, String> {
        log_db_operation_start("get_workflow", "workflows");

        let row = sqlx::query("SELECT data FROM workflows WHERE id = ?")
            .bind(id)
            .fetch_optional(self.pool())
            .await
            .map_err(|e| {
                log_db_operation_error("get_workflow", "workflows", &e.to_string());
                format!("Database error: {}", e)
            })?;

        match row {
            Some(row) => {
                let json_data: String = row.get("data");
                log_deserialization("WorkflowDefinition", json_data.len());

                let workflow = serde_json::from_str(&json_data).map_err(|e| {
                    log_db_operation_error("get_workflow", "workflows", &e.to_string());
                    format!("Deserialization error: {}", e)
                })?;

                log_db_operation_success("get_workflow", "workflows", 0);
                Ok(Some(workflow))
            }
            None => {
                log_db_operation_success("get_workflow", "workflows", 0);
                Ok(None)
            }
        }
    }

    /// List all workflow definitions
    pub async fn list_workflows(&self) -> Result<Vec<WorkflowDefinition>, String> {
        log_db_operation_start("list_workflows", "workflows");

        let rows = sqlx::query(
            "SELECT data FROM workflows ORDER BY json_extract(data, '$.created_at') DESC",
        )
        .fetch_all(self.pool())
        .await
        .map_err(|e| {
            log_db_operation_error("list_workflows", "workflows", &e.to_string());
            format!("Database error: {}", e)
        })?;

        let mut workflows = Vec::new();
        for row in rows {
            let json_data: String = row.get("data");
            log_deserialization("WorkflowDefinition", json_data.len());

            let workflow = serde_json::from_str(&json_data).map_err(|e| {
                log_db_operation_error("list_workflows", "workflows", &e.to_string());
                format!("Deserialization error: {}", e)
            })?;
            workflows.push(workflow);
        }

        log_db_operation_success("list_workflows", "workflows", 0);
        Ok(workflows)
    }

    /// Delete a workflow definition
    pub async fn delete_workflow(&self, id: &str) -> Result<(), String> {
        log_db_operation_start("delete_workflow", "workflows");

        sqlx::query("DELETE FROM workflows WHERE id = ?")
            .bind(id)
            .execute(self.pool())
            .await
            .map_err(|e| {
                log_db_operation_error("delete_workflow", "workflows", &e.to_string());
                format!("Database error: {}", e)
            })?;

        log_db_operation_success("delete_workflow", "workflows", 0);
        Ok(())
    }
}

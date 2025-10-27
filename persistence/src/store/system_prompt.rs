//! System prompt store operations
//!
//! This file contains ONLY system prompt CRUD operations following Single Responsibility Principle.

use super::Store;
use crate::logging::{
    log_db_operation_error, log_db_operation_start, log_db_operation_success, log_deserialization,
    log_serialization,
};
use crate::models::SystemPrompt;
use sqlx::Row;

impl Store {
    /// Save a system prompt
    pub async fn save_system_prompt(&self, prompt: &SystemPrompt) -> Result<(), String> {
        log_db_operation_start("save_system_prompt", "system_prompts");

        let json_data = serde_json::to_string(prompt).map_err(|e| {
            log_db_operation_error("save_system_prompt", "system_prompts", &e.to_string());
            format!("Serialization error: {}", e)
        })?;

        log_serialization("SystemPrompt", json_data.len());

        sqlx::query("INSERT OR REPLACE INTO system_prompts (id, data) VALUES (?, ?)")
            .bind(&prompt.id)
            .bind(&json_data)
            .execute(self.pool())
            .await
            .map_err(|e| {
                log_db_operation_error("save_system_prompt", "system_prompts", &e.to_string());
                format!("Database error: {}", e)
            })?;

        log_db_operation_success("save_system_prompt", "system_prompts", 0);
        Ok(())
    }

    /// Get a system prompt by ID
    pub async fn get_system_prompt(&self, id: &str) -> Result<Option<SystemPrompt>, String> {
        log_db_operation_start("get_system_prompt", "system_prompts");

        let row = sqlx::query("SELECT data FROM system_prompts WHERE id = ?")
            .bind(id)
            .fetch_optional(self.pool())
            .await
            .map_err(|e| {
                log_db_operation_error("get_system_prompt", "system_prompts", &e.to_string());
                format!("Database error: {}", e)
            })?;

        match row {
            Some(row) => {
                let json_data: String = row.get("data");
                log_deserialization("SystemPrompt", json_data.len());

                let prompt = serde_json::from_str(&json_data).map_err(|e| {
                    log_db_operation_error("get_system_prompt", "system_prompts", &e.to_string());
                    format!("Deserialization error: {}", e)
                })?;
                Ok(Some(prompt))
            }
            None => Ok(None),
        }
    }

    /// List all system prompts
    pub async fn list_system_prompts(&self) -> Result<Vec<SystemPrompt>, String> {
        log_db_operation_start("list_system_prompts", "system_prompts");

        let rows = sqlx::query("SELECT data FROM system_prompts ORDER BY id")
            .fetch_all(self.pool())
            .await
            .map_err(|e| {
                log_db_operation_error("list_system_prompts", "system_prompts", &e.to_string());
                format!("Database error: {}", e)
            })?;

        let mut prompts = Vec::new();
        for row in rows {
            let json_data: String = row.get("data");
            log_deserialization("SystemPrompt", json_data.len());

            let prompt = serde_json::from_str(&json_data).map_err(|e| {
                log_db_operation_error("list_system_prompts", "system_prompts", &e.to_string());
                format!("Deserialization error: {}", e)
            })?;
            prompts.push(prompt);
        }

        log_db_operation_success(
            "list_system_prompts",
            "system_prompts",
            prompts.len() as u64,
        );
        Ok(prompts)
    }

    /// Delete a system prompt
    pub async fn delete_system_prompt(&self, id: &str) -> Result<(), String> {
        log_db_operation_start("delete_system_prompt", "system_prompts");

        sqlx::query("DELETE FROM system_prompts WHERE id = ?")
            .bind(id)
            .execute(self.pool())
            .await
            .map_err(|e| {
                log_db_operation_error("delete_system_prompt", "system_prompts", &e.to_string());
                format!("Database error: {}", e)
            })?;

        log_db_operation_success("delete_system_prompt", "system_prompts", 0);
        Ok(())
    }

    /// Clear all system prompts (useful for reloading)
    pub async fn clear_system_prompts(&self) -> Result<(), String> {
        log_db_operation_start("clear_system_prompts", "system_prompts");

        sqlx::query("DELETE FROM system_prompts")
            .execute(self.pool())
            .await
            .map_err(|e| {
                log_db_operation_error("clear_system_prompts", "system_prompts", &e.to_string());
                format!("Database error: {}", e)
            })?;

        log_db_operation_success("clear_system_prompts", "system_prompts", 0);
        Ok(())
    }
}

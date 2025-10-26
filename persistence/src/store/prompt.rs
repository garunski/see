//! Prompt store operations
//!
//! This file contains ONLY prompt operations following Single Responsibility Principle.

use super::Store;
use crate::logging::{
    log_db_operation_error, log_db_operation_start, log_db_operation_success, log_deserialization,
    log_serialization,
};
use crate::models::UserPrompt;
use sqlx::Row;

impl Store {
    /// Save a user prompt
    pub async fn save_prompt(&self, prompt: &UserPrompt) -> Result<(), String> {
        log_db_operation_start("save_prompt", "user_prompts");

        let json_data = serde_json::to_string(prompt).map_err(|e| {
            log_db_operation_error("save_prompt", "user_prompts", &e.to_string());
            format!("Serialization error: {}", e)
        })?;

        log_serialization("UserPrompt", json_data.len());

        sqlx::query("INSERT OR REPLACE INTO user_prompts (id, data) VALUES (?, ?)")
            .bind(&prompt.id)
            .bind(&json_data)
            .execute(self.pool())
            .await
            .map_err(|e| {
                log_db_operation_error("save_prompt", "user_prompts", &e.to_string());
                format!("Database error: {}", e)
            })?;

        log_db_operation_success("save_prompt", "user_prompts", 0);
        Ok(())
    }

    /// List all user prompts
    pub async fn list_prompts(&self) -> Result<Vec<UserPrompt>, String> {
        log_db_operation_start("list_prompts", "user_prompts");

        let rows = sqlx::query("SELECT data FROM user_prompts ORDER BY id")
            .fetch_all(self.pool())
            .await
            .map_err(|e| {
                log_db_operation_error("list_prompts", "user_prompts", &e.to_string());
                format!("Database error: {}", e)
            })?;

        let mut prompts = Vec::new();
        for row in rows {
            let json_data: String = row.get("data");
            log_deserialization("UserPrompt", json_data.len());

            let prompt = serde_json::from_str(&json_data).map_err(|e| {
                log_db_operation_error("list_prompts", "user_prompts", &e.to_string());
                format!("Deserialization error: {}", e)
            })?;
            prompts.push(prompt);
        }

        log_db_operation_success("list_prompts", "user_prompts", 0);
        Ok(prompts)
    }

    /// Delete a user prompt
    pub async fn delete_prompt(&self, id: &str) -> Result<(), String> {
        log_db_operation_start("delete_prompt", "user_prompts");

        sqlx::query("DELETE FROM user_prompts WHERE id = ?")
            .bind(id)
            .execute(self.pool())
            .await
            .map_err(|e| {
                log_db_operation_error("delete_prompt", "user_prompts", &e.to_string());
                format!("Database error: {}", e)
            })?;

        log_db_operation_success("delete_prompt", "user_prompts", 0);
        Ok(())
    }
}

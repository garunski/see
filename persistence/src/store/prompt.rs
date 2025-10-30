use super::Store;
use crate::logging::{
    log_db_operation_error, log_db_operation_start, log_db_operation_success, log_deserialization,
    log_serialization,
};
use crate::models::Prompt;
use sqlx::Row;

impl Store {
    pub async fn save_prompt(&self, prompt: &Prompt) -> Result<(), String> {
        log_db_operation_start("save_prompt", "prompts");

        let json_data = serde_json::to_string(prompt).map_err(|e| {
            log_db_operation_error("save_prompt", "prompts", &e.to_string());
            format!("Serialization error: {}", e)
        })?;

        log_serialization("Prompt", json_data.len());

        sqlx::query("INSERT OR REPLACE INTO prompts (id, data) VALUES (?, ?)")
            .bind(&prompt.id)
            .bind(&json_data)
            .execute(self.pool())
            .await
            .map_err(|e| {
                log_db_operation_error("save_prompt", "prompts", &e.to_string());
                format!("Database error: {}", e)
            })?;

        log_db_operation_success("save_prompt", "prompts", 0);
        Ok(())
    }

    pub async fn list_prompts(&self) -> Result<Vec<Prompt>, String> {
        log_db_operation_start("list_prompts", "prompts");

        let rows = sqlx::query("SELECT data FROM prompts ORDER BY id")
            .fetch_all(self.pool())
            .await
            .map_err(|e| {
                log_db_operation_error("list_prompts", "prompts", &e.to_string());
                format!("Database error: {}", e)
            })?;

        let mut prompts = Vec::new();
        for row in rows {
            let json_data: String = row.get("data");
            log_deserialization("Prompt", json_data.len());

            let prompt = serde_json::from_str(&json_data).map_err(|e| {
                log_db_operation_error("list_prompts", "prompts", &e.to_string());
                format!("Deserialization error: {}", e)
            })?;
            prompts.push(prompt);
        }

        log_db_operation_success("list_prompts", "prompts", 0);
        Ok(prompts)
    }

    pub async fn delete_prompt(&self, id: &str) -> Result<(), String> {
        log_db_operation_start("delete_prompt", "prompts");

        sqlx::query("DELETE FROM prompts WHERE id = ?")
            .bind(id)
            .execute(self.pool())
            .await
            .map_err(|e| {
                log_db_operation_error("delete_prompt", "prompts", &e.to_string());
                format!("Database error: {}", e)
            })?;

        log_db_operation_success("delete_prompt", "prompts", 0);
        Ok(())
    }
}

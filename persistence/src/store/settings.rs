//! Settings store operations
//! 
//! This file contains ONLY settings operations following Single Responsibility Principle.

use sqlx::Row;
use crate::models::AppSettings;
use crate::logging::{log_db_operation_start, log_db_operation_success, log_db_operation_error, log_serialization, log_deserialization};
use super::Store;

impl Store {
    /// Load application settings
    pub async fn load_settings(&self) -> Result<Option<AppSettings>, String> {
        log_db_operation_start("load_settings", "settings");
        
        let row = sqlx::query("SELECT data FROM settings WHERE id = 'app_settings'")
            .fetch_optional(self.pool())
            .await
            .map_err(|e| {
                log_db_operation_error("load_settings", "settings", &e.to_string());
                format!("Database error: {}", e)
            })?;

        match row {
            Some(row) => {
                let json_data: String = row.get("data");
                log_deserialization("AppSettings", json_data.len());
                
                let settings = serde_json::from_str(&json_data)
                    .map_err(|e| {
                        log_db_operation_error("load_settings", "settings", &e.to_string());
                        format!("Deserialization error: {}", e)
                    })?;
                
                log_db_operation_success("load_settings", "settings", 0);
                Ok(Some(settings))
            }
            None => {
                log_db_operation_success("load_settings", "settings", 0);
                Ok(None)
            }
        }
    }

    /// Save application settings
    pub async fn save_settings(&self, settings: &AppSettings) -> Result<(), String> {
        log_db_operation_start("save_settings", "settings");
        
        let json_data = serde_json::to_string(settings)
            .map_err(|e| {
                log_db_operation_error("save_settings", "settings", &e.to_string());
                format!("Serialization error: {}", e)
            })?;
        
        log_serialization("AppSettings", json_data.len());
        
        sqlx::query("INSERT OR REPLACE INTO settings (id, data) VALUES ('app_settings', ?)")
            .bind(&json_data)
            .execute(self.pool())
            .await
            .map_err(|e| {
                log_db_operation_error("save_settings", "settings", &e.to_string());
                format!("Database error: {}", e)
            })?;
        
        log_db_operation_success("save_settings", "settings", 0);
        Ok(())
    }
}

//! Settings store for CRUD operations

use std::sync::Arc;
use rusqlite::params;
use chrono::Utc;
use serde_json;

use crate::db::DatabasePool;
use crate::error::PersistenceError;
use crate::models::Setting;
use tracing::{debug, info, warn};

/// Store for settings operations
pub struct SettingsStore {
    pool: Arc<DatabasePool>,
}

impl SettingsStore {
    /// Create a new settings store
    pub fn new(pool: Arc<DatabasePool>) -> Self {
        Self { pool }
    }
    
    /// Save or update a setting (upsert)
    pub async fn save(&self, setting: &Setting) -> Result<(), PersistenceError> {
        debug!("Saving setting: {}", setting.key);
        
        let conn = self.pool.get_connection()?;
        let now = Utc::now().to_rfc3339();
        let data = serde_json::to_string(setting)
            .map_err(|e| PersistenceError::Serialization(format!("Failed to serialize setting: {}", e)))?;
        
        conn.execute(
            "INSERT OR REPLACE INTO settings (key, data, instance_id, created_at, updated_at) 
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                setting.key,
                data,
                setting.metadata.get("instance_id").and_then(|v| v.as_str()),
                now,
                now
            ],
        )?;
        
        info!("Setting saved successfully: {}", setting.key);
        Ok(())
    }
    
    /// Get a setting by key
    pub async fn get(&self, key: &str) -> Result<Option<Setting>, PersistenceError> {
        debug!("Getting setting: {}", key);
        
        let conn = self.pool.get_connection()?;
        let mut stmt = conn.prepare("SELECT data FROM settings WHERE key = ?1")?;
        let mut rows = stmt.query_map([key], |row| {
            let data: String = row.get(0)?;
            Ok(data)
        })?;
        
        if let Some(row) = rows.next() {
            let data = row?;
            let setting: Setting = serde_json::from_str(&data)
                .map_err(|e| PersistenceError::Serialization(format!("Failed to deserialize setting: {}", e)))?;
            debug!("Setting found: {}", setting.key);
            Ok(Some(setting))
        } else {
            debug!("Setting not found: {}", key);
            Ok(None)
        }
    }
    
    /// List all settings
    pub async fn list(&self) -> Result<Vec<Setting>, PersistenceError> {
        debug!("Listing all settings");
        
        let conn = self.pool.get_connection()?;
        let mut stmt = conn.prepare("SELECT data FROM settings ORDER BY key ASC")?;
        let rows = stmt.query_map([], |row| {
            let data: String = row.get(0)?;
            Ok(data)
        })?;
        
        let mut settings = Vec::new();
        for row in rows {
            let data = row?;
            let setting: Setting = serde_json::from_str(&data)
                .map_err(|e| PersistenceError::Serialization(format!("Failed to deserialize setting: {}", e)))?;
            settings.push(setting);
        }
        
        debug!("Found {} settings", settings.len());
        Ok(settings)
    }
    
    /// List settings by instance
    pub async fn list_by_instance(&self, instance_id: &str) -> Result<Vec<Setting>, PersistenceError> {
        debug!("Listing settings for instance: {}", instance_id);
        
        let conn = self.pool.get_connection()?;
        let mut stmt = conn.prepare(
            "SELECT data FROM settings WHERE instance_id = ?1 ORDER BY key ASC"
        )?;
        let rows = stmt.query_map([instance_id], |row| {
            let data: String = row.get(0)?;
            Ok(data)
        })?;
        
        let mut settings = Vec::new();
        for row in rows {
            let data = row?;
            let setting: Setting = serde_json::from_str(&data)
                .map_err(|e| PersistenceError::Serialization(format!("Failed to deserialize setting: {}", e)))?;
            settings.push(setting);
        }
        
        debug!("Found {} settings for instance {}", settings.len(), instance_id);
        Ok(settings)
    }
    
    /// Get setting value as string
    pub async fn get_string(&self, key: &str) -> Result<Option<String>, PersistenceError> {
        debug!("Getting setting as string: {}", key);
        
        if let Some(setting) = self.get(key).await? {
            Ok(setting.get_value_as_string())
        } else {
            Ok(None)
        }
    }
    
    /// Get setting value as boolean
    pub async fn get_bool(&self, key: &str) -> Result<Option<bool>, PersistenceError> {
        debug!("Getting setting as boolean: {}", key);
        
        if let Some(setting) = self.get(key).await? {
            Ok(setting.get_value_as_bool())
        } else {
            Ok(None)
        }
    }
    
    /// Get setting value as number
    pub async fn get_number(&self, key: &str) -> Result<Option<f64>, PersistenceError> {
        debug!("Getting setting as number: {}", key);
        
        if let Some(setting) = self.get(key).await? {
            Ok(setting.get_value_as_number())
        } else {
            Ok(None)
        }
    }
    
    /// Set a string setting
    pub async fn set_string(&self, key: &str, value: &str) -> Result<(), PersistenceError> {
        debug!("Setting string value for key: {}", key);
        
        let setting = Setting::new(key.to_string(), serde_json::Value::String(value.to_string()));
        self.save(&setting).await
    }
    
    /// Set a boolean setting
    pub async fn set_bool(&self, key: &str, value: bool) -> Result<(), PersistenceError> {
        debug!("Setting boolean value for key: {}", key);
        
        let setting = Setting::new(key.to_string(), serde_json::Value::Bool(value));
        self.save(&setting).await
    }
    
    /// Set a number setting
    pub async fn set_number(&self, key: &str, value: f64) -> Result<(), PersistenceError> {
        debug!("Setting number value for key: {}", key);
        
        let setting = Setting::new(key.to_string(), serde_json::Value::Number(serde_json::Number::from_f64(value).unwrap()));
        self.save(&setting).await
    }
    
    /// Set a JSON value setting
    pub async fn set_json(&self, key: &str, value: serde_json::Value) -> Result<(), PersistenceError> {
        debug!("Setting JSON value for key: {}", key);
        
        let setting = Setting::new(key.to_string(), value);
        self.save(&setting).await
    }
    
    /// Delete a setting
    pub async fn delete(&self, key: &str) -> Result<(), PersistenceError> {
        debug!("Deleting setting: {}", key);
        
        let conn = self.pool.get_connection()?;
        let changes = conn.execute("DELETE FROM settings WHERE key = ?1", [key])?;
        
        if changes > 0 {
            info!("Setting deleted successfully: {}", key);
        } else {
            warn!("Setting not found for deletion: {}", key);
        }
        
        Ok(())
    }
    
    /// Count total settings
    pub async fn count(&self) -> Result<usize, PersistenceError> {
        debug!("Counting total settings");
        
        let conn = self.pool.get_connection()?;
        let mut stmt = conn.prepare("SELECT COUNT(*) FROM settings")?;
        let count: i64 = stmt.query_row([], |row| row.get(0))?;
        
        debug!("Total settings: {}", count);
        Ok(count as usize)
    }
}

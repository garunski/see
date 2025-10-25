//! User prompt store for CRUD operations

use std::sync::Arc;
use rusqlite::params;
use chrono::Utc;
use serde_json;

use crate::db::DatabasePool;
use crate::error::PersistenceError;
use crate::models::UserPrompt;
use tracing::{debug, info, warn};

/// Store for user prompt operations
pub struct UserPromptStore {
    pool: Arc<DatabasePool>,
}

impl UserPromptStore {
    /// Create a new user prompt store
    pub fn new(pool: Arc<DatabasePool>) -> Self {
        Self { pool }
    }
    
    /// Save or update a user prompt (upsert)
    pub async fn save(&self, prompt: &UserPrompt) -> Result<(), PersistenceError> {
        debug!("Saving user prompt: {}", prompt.name);
        
        let conn = self.pool.get_connection()?;
        let now = Utc::now().to_rfc3339();
        let data = serde_json::to_string(prompt)
            .map_err(|e| PersistenceError::Serialization(format!("Failed to serialize user prompt: {}", e)))?;
        
        conn.execute(
            "INSERT OR REPLACE INTO user_prompts (id, data, instance_id, created_at, updated_at) 
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                prompt.id,
                data,
                prompt.metadata.get("instance_id").and_then(|v| v.as_str()),
                now,
                now
            ],
        )?;
        
        info!("User prompt saved successfully: {}", prompt.id);
        Ok(())
    }
    
    /// Get a user prompt by ID
    pub async fn get(&self, id: &str) -> Result<Option<UserPrompt>, PersistenceError> {
        debug!("Getting user prompt: {}", id);
        
        let conn = self.pool.get_connection()?;
        let mut stmt = conn.prepare("SELECT data FROM user_prompts WHERE id = ?1")?;
        let mut rows = stmt.query_map([id], |row| {
            let data: String = row.get(0)?;
            Ok(data)
        })?;
        
        if let Some(row) = rows.next() {
            let data = row?;
            let prompt: UserPrompt = serde_json::from_str(&data)
                .map_err(|e| PersistenceError::Serialization(format!("Failed to deserialize user prompt: {}", e)))?;
            debug!("User prompt found: {}", prompt.name);
            Ok(Some(prompt))
        } else {
            debug!("User prompt not found: {}", id);
            Ok(None)
        }
    }
    
    /// List all user prompts
    pub async fn list(&self) -> Result<Vec<UserPrompt>, PersistenceError> {
        debug!("Listing all user prompts");
        
        let conn = self.pool.get_connection()?;
        let mut stmt = conn.prepare("SELECT data FROM user_prompts ORDER BY created_at DESC")?;
        let rows = stmt.query_map([], |row| {
            let data: String = row.get(0)?;
            Ok(data)
        })?;
        
        let mut prompts = Vec::new();
        for row in rows {
            let data = row?;
            let prompt: UserPrompt = serde_json::from_str(&data)
                .map_err(|e| PersistenceError::Serialization(format!("Failed to deserialize user prompt: {}", e)))?;
            prompts.push(prompt);
        }
        
        debug!("Found {} user prompts", prompts.len());
        Ok(prompts)
    }
    
    /// List user prompts by instance
    pub async fn list_by_instance(&self, instance_id: &str) -> Result<Vec<UserPrompt>, PersistenceError> {
        debug!("Listing user prompts for instance: {}", instance_id);
        
        let conn = self.pool.get_connection()?;
        let mut stmt = conn.prepare(
            "SELECT data FROM user_prompts WHERE instance_id = ?1 ORDER BY created_at DESC"
        )?;
        let rows = stmt.query_map([instance_id], |row| {
            let data: String = row.get(0)?;
            Ok(data)
        })?;
        
        let mut prompts = Vec::new();
        for row in rows {
            let data = row?;
            let prompt: UserPrompt = serde_json::from_str(&data)
                .map_err(|e| PersistenceError::Serialization(format!("Failed to deserialize user prompt: {}", e)))?;
            prompts.push(prompt);
        }
        
        debug!("Found {} user prompts for instance {}", prompts.len(), instance_id);
        Ok(prompts)
    }
    
    /// Search user prompts by name
    pub async fn search_by_name(&self, name_pattern: &str) -> Result<Vec<UserPrompt>, PersistenceError> {
        debug!("Searching user prompts by name pattern: {}", name_pattern);
        
        let conn = self.pool.get_connection()?;
        let pattern = format!("%{}%", name_pattern);
        let mut stmt = conn.prepare(
            "SELECT data FROM user_prompts WHERE json_extract(data, '$.name') LIKE ?1 ORDER BY created_at DESC"
        )?;
        let rows = stmt.query_map([&pattern], |row| {
            let data: String = row.get(0)?;
            Ok(data)
        })?;
        
        let mut prompts = Vec::new();
        for row in rows {
            let data = row?;
            let prompt: UserPrompt = serde_json::from_str(&data)
                .map_err(|e| PersistenceError::Serialization(format!("Failed to deserialize user prompt: {}", e)))?;
            prompts.push(prompt);
        }
        
        debug!("Found {} user prompts matching pattern '{}'", prompts.len(), name_pattern);
        Ok(prompts)
    }
    
    /// Delete a user prompt
    pub async fn delete(&self, id: &str) -> Result<(), PersistenceError> {
        debug!("Deleting user prompt: {}", id);
        
        let conn = self.pool.get_connection()?;
        let changes = conn.execute("DELETE FROM user_prompts WHERE id = ?1", [id])?;
        
        if changes > 0 {
            info!("User prompt deleted successfully: {}", id);
        } else {
            warn!("User prompt not found for deletion: {}", id);
        }
        
        Ok(())
    }
    
    /// Count total user prompts
    pub async fn count(&self) -> Result<usize, PersistenceError> {
        debug!("Counting total user prompts");
        
        let conn = self.pool.get_connection()?;
        let mut stmt = conn.prepare("SELECT COUNT(*) FROM user_prompts")?;
        let count: i64 = stmt.query_row([], |row| row.get(0))?;
        
        debug!("Total user prompts: {}", count);
        Ok(count as usize)
    }
}

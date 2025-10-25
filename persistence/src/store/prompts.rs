//! Prompt store for CRUD operations

use std::sync::Arc;
use rusqlite::params;
use chrono::Utc;
use serde_json;

use crate::db::DatabasePool;
use crate::error::PersistenceError;
use crate::models::Prompt;
use tracing::{debug, info, warn};

/// Store for prompt operations
pub struct PromptStore {
    pool: Arc<DatabasePool>,
}

impl PromptStore {
    /// Create a new prompt store
    pub fn new(pool: Arc<DatabasePool>) -> Self {
        Self { pool }
    }
    
    /// Save or update a prompt (upsert)
    pub async fn save(&self, prompt: &Prompt) -> Result<(), PersistenceError> {
        debug!("Saving prompt: {}", prompt.name);
        
        let conn = self.pool.get_connection()?;
        let now = Utc::now().to_rfc3339();
        let data = serde_json::to_string(prompt)
            .map_err(|e| PersistenceError::Serialization(format!("Failed to serialize prompt: {}", e)))?;
        
        conn.execute(
            "INSERT OR REPLACE INTO prompts (id, data, instance_id, created_at, updated_at) 
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                prompt.id,
                data,
                prompt.metadata.get("instance_id").and_then(|v| v.as_str()),
                now,
                now
            ],
        )?;
        
        info!("Prompt saved successfully: {}", prompt.id);
        Ok(())
    }
    
    /// Get a prompt by ID
    pub async fn get(&self, id: &str) -> Result<Option<Prompt>, PersistenceError> {
        debug!("Getting prompt: {}", id);
        
        let conn = self.pool.get_connection()?;
        let mut stmt = conn.prepare("SELECT data FROM prompts WHERE id = ?1")?;
        let mut rows = stmt.query_map([id], |row| {
            let data: String = row.get(0)?;
            Ok(data)
        })?;
        
        if let Some(row) = rows.next() {
            let data = row?;
            let prompt: Prompt = serde_json::from_str(&data)
                .map_err(|e| PersistenceError::Serialization(format!("Failed to deserialize prompt: {}", e)))?;
            debug!("Prompt found: {}", prompt.name);
            Ok(Some(prompt))
        } else {
            debug!("Prompt not found: {}", id);
            Ok(None)
        }
    }
    
    /// List all prompts
    pub async fn list(&self) -> Result<Vec<Prompt>, PersistenceError> {
        debug!("Listing all prompts");
        
        let conn = self.pool.get_connection()?;
        let mut stmt = conn.prepare("SELECT data FROM prompts ORDER BY created_at DESC")?;
        let rows = stmt.query_map([], |row| {
            let data: String = row.get(0)?;
            Ok(data)
        })?;
        
        let mut prompts = Vec::new();
        for row in rows {
            let data = row?;
            let prompt: Prompt = serde_json::from_str(&data)
                .map_err(|e| PersistenceError::Serialization(format!("Failed to deserialize prompt: {}", e)))?;
            prompts.push(prompt);
        }
        
        debug!("Found {} prompts", prompts.len());
        Ok(prompts)
    }
    
    /// List prompts by instance
    pub async fn list_by_instance(&self, instance_id: &str) -> Result<Vec<Prompt>, PersistenceError> {
        debug!("Listing prompts for instance: {}", instance_id);
        
        let conn = self.pool.get_connection()?;
        let mut stmt = conn.prepare(
            "SELECT data FROM prompts WHERE instance_id = ?1 ORDER BY created_at DESC"
        )?;
        let rows = stmt.query_map([instance_id], |row| {
            let data: String = row.get(0)?;
            Ok(data)
        })?;
        
        let mut prompts = Vec::new();
        for row in rows {
            let data = row?;
            let prompt: Prompt = serde_json::from_str(&data)
                .map_err(|e| PersistenceError::Serialization(format!("Failed to deserialize prompt: {}", e)))?;
            prompts.push(prompt);
        }
        
        debug!("Found {} prompts for instance {}", prompts.len(), instance_id);
        Ok(prompts)
    }
    
    /// Search prompts by name
    pub async fn search_by_name(&self, name_pattern: &str) -> Result<Vec<Prompt>, PersistenceError> {
        debug!("Searching prompts by name pattern: {}", name_pattern);
        
        let conn = self.pool.get_connection()?;
        let pattern = format!("%{}%", name_pattern);
        let mut stmt = conn.prepare(
            "SELECT data FROM prompts WHERE json_extract(data, '$.name') LIKE ?1 ORDER BY created_at DESC"
        )?;
        let rows = stmt.query_map([&pattern], |row| {
            let data: String = row.get(0)?;
            Ok(data)
        })?;
        
        let mut prompts = Vec::new();
        for row in rows {
            let data = row?;
            let prompt: Prompt = serde_json::from_str(&data)
                .map_err(|e| PersistenceError::Serialization(format!("Failed to deserialize prompt: {}", e)))?;
            prompts.push(prompt);
        }
        
        debug!("Found {} prompts matching pattern '{}'", prompts.len(), name_pattern);
        Ok(prompts)
    }
    
    /// Search prompts by tag
    pub async fn search_by_tag(&self, tag: &str) -> Result<Vec<Prompt>, PersistenceError> {
        debug!("Searching prompts by tag: {}", tag);
        
        let conn = self.pool.get_connection()?;
        let mut stmt = conn.prepare(
            "SELECT data FROM prompts WHERE json_extract(data, '$.tags') LIKE ?1 ORDER BY created_at DESC"
        )?;
        let pattern = format!("%\"{}\"%", tag);
        let rows = stmt.query_map([&pattern], |row| {
            let data: String = row.get(0)?;
            Ok(data)
        })?;
        
        let mut prompts = Vec::new();
        for row in rows {
            let data = row?;
            let prompt: Prompt = serde_json::from_str(&data)
                .map_err(|e| PersistenceError::Serialization(format!("Failed to deserialize prompt: {}", e)))?;
            prompts.push(prompt);
        }
        
        debug!("Found {} prompts with tag '{}'", prompts.len(), tag);
        Ok(prompts)
    }
    
    /// Delete a prompt
    pub async fn delete(&self, id: &str) -> Result<(), PersistenceError> {
        debug!("Deleting prompt: {}", id);
        
        let conn = self.pool.get_connection()?;
        let changes = conn.execute("DELETE FROM prompts WHERE id = ?1", [id])?;
        
        if changes > 0 {
            info!("Prompt deleted successfully: {}", id);
        } else {
            warn!("Prompt not found for deletion: {}", id);
        }
        
        Ok(())
    }
    
    /// Count total prompts
    pub async fn count(&self) -> Result<usize, PersistenceError> {
        debug!("Counting total prompts");
        
        let conn = self.pool.get_connection()?;
        let mut stmt = conn.prepare("SELECT COUNT(*) FROM prompts")?;
        let count: i64 = stmt.query_row([], |row| row.get(0))?;
        
        debug!("Total prompts: {}", count);
        Ok(count as usize)
    }
}

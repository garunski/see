//! AI prompt store for CRUD operations

use chrono::Utc;
use rusqlite::params;
use serde_json;
use std::sync::Arc;

use crate::db::DatabasePool;
use crate::error::PersistenceError;
use crate::models::AiPrompt;
use tracing::{debug, info, warn};

/// Store for AI prompt operations
pub struct AiPromptStore {
    pool: Arc<DatabasePool>,
}

impl AiPromptStore {
    /// Create a new AI prompt store
    pub fn new(pool: Arc<DatabasePool>) -> Self {
        Self { pool }
    }

    /// Save or update an AI prompt (upsert)
    pub async fn save(&self, prompt: &AiPrompt) -> Result<(), PersistenceError> {
        debug!("Saving AI prompt: {}", prompt.name);

        let conn = self.pool.get_connection()?;
        let now = Utc::now().to_rfc3339();
        let data = serde_json::to_string(prompt).map_err(|e| {
            PersistenceError::Serialization(format!("Failed to serialize AI prompt: {}", e))
        })?;

        conn.execute(
            "INSERT OR REPLACE INTO ai_prompts (id, data, instance_id, created_at, updated_at) 
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                prompt.id,
                data,
                prompt.metadata.get("instance_id").and_then(|v| v.as_str()),
                now,
                now
            ],
        )?;

        info!("AI prompt saved successfully: {}", prompt.id);
        Ok(())
    }

    /// Get an AI prompt by ID
    pub async fn get(&self, id: &str) -> Result<Option<AiPrompt>, PersistenceError> {
        debug!("Getting AI prompt: {}", id);

        let conn = self.pool.get_connection()?;
        let mut stmt = conn.prepare("SELECT data FROM ai_prompts WHERE id = ?1")?;
        let mut rows = stmt.query_map([id], |row| {
            let data: String = row.get(0)?;
            Ok(data)
        })?;

        if let Some(row) = rows.next() {
            let data = row?;
            let prompt: AiPrompt = serde_json::from_str(&data).map_err(|e| {
                PersistenceError::Serialization(format!("Failed to deserialize AI prompt: {}", e))
            })?;
            debug!("AI prompt found: {}", prompt.name);
            Ok(Some(prompt))
        } else {
            debug!("AI prompt not found: {}", id);
            Ok(None)
        }
    }

    /// List all AI prompts
    pub async fn list(&self) -> Result<Vec<AiPrompt>, PersistenceError> {
        debug!("Listing all AI prompts");

        let conn = self.pool.get_connection()?;
        let mut stmt = conn.prepare("SELECT data FROM ai_prompts ORDER BY created_at DESC")?;
        let rows = stmt.query_map([], |row| {
            let data: String = row.get(0)?;
            Ok(data)
        })?;

        let mut prompts = Vec::new();
        for row in rows {
            let data = row?;
            let prompt: AiPrompt = serde_json::from_str(&data).map_err(|e| {
                PersistenceError::Serialization(format!("Failed to deserialize AI prompt: {}", e))
            })?;
            prompts.push(prompt);
        }

        debug!("Found {} AI prompts", prompts.len());
        Ok(prompts)
    }

    /// List AI prompts by instance
    pub async fn list_by_instance(
        &self,
        instance_id: &str,
    ) -> Result<Vec<AiPrompt>, PersistenceError> {
        debug!("Listing AI prompts for instance: {}", instance_id);

        let conn = self.pool.get_connection()?;
        let mut stmt = conn.prepare(
            "SELECT data FROM ai_prompts WHERE instance_id = ?1 ORDER BY created_at DESC",
        )?;
        let rows = stmt.query_map([instance_id], |row| {
            let data: String = row.get(0)?;
            Ok(data)
        })?;

        let mut prompts = Vec::new();
        for row in rows {
            let data = row?;
            let prompt: AiPrompt = serde_json::from_str(&data).map_err(|e| {
                PersistenceError::Serialization(format!("Failed to deserialize AI prompt: {}", e))
            })?;
            prompts.push(prompt);
        }

        debug!(
            "Found {} AI prompts for instance {}",
            prompts.len(),
            instance_id
        );
        Ok(prompts)
    }

    /// Search AI prompts by name
    pub async fn search_by_name(
        &self,
        name_pattern: &str,
    ) -> Result<Vec<AiPrompt>, PersistenceError> {
        debug!("Searching AI prompts by name pattern: {}", name_pattern);

        let conn = self.pool.get_connection()?;
        let pattern = format!("%{}%", name_pattern);
        let mut stmt = conn.prepare(
            "SELECT data FROM ai_prompts WHERE json_extract(data, '$.name') LIKE ?1 ORDER BY created_at DESC"
        )?;
        let rows = stmt.query_map([&pattern], |row| {
            let data: String = row.get(0)?;
            Ok(data)
        })?;

        let mut prompts = Vec::new();
        for row in rows {
            let data = row?;
            let prompt: AiPrompt = serde_json::from_str(&data).map_err(|e| {
                PersistenceError::Serialization(format!("Failed to deserialize AI prompt: {}", e))
            })?;
            prompts.push(prompt);
        }

        debug!(
            "Found {} AI prompts matching pattern '{}'",
            prompts.len(),
            name_pattern
        );
        Ok(prompts)
    }

    /// Search AI prompts by model
    pub async fn search_by_model(&self, model: &str) -> Result<Vec<AiPrompt>, PersistenceError> {
        debug!("Searching AI prompts by model: {}", model);

        let conn = self.pool.get_connection()?;
        let mut stmt = conn.prepare(
            "SELECT data FROM ai_prompts WHERE json_extract(data, '$.model') = ?1 ORDER BY created_at DESC"
        )?;
        let rows = stmt.query_map([model], |row| {
            let data: String = row.get(0)?;
            Ok(data)
        })?;

        let mut prompts = Vec::new();
        for row in rows {
            let data = row?;
            let prompt: AiPrompt = serde_json::from_str(&data).map_err(|e| {
                PersistenceError::Serialization(format!("Failed to deserialize AI prompt: {}", e))
            })?;
            prompts.push(prompt);
        }

        debug!("Found {} AI prompts for model '{}'", prompts.len(), model);
        Ok(prompts)
    }

    /// Delete an AI prompt
    pub async fn delete(&self, id: &str) -> Result<(), PersistenceError> {
        debug!("Deleting AI prompt: {}", id);

        let conn = self.pool.get_connection()?;
        let changes = conn.execute("DELETE FROM ai_prompts WHERE id = ?1", [id])?;

        if changes > 0 {
            info!("AI prompt deleted successfully: {}", id);
        } else {
            warn!("AI prompt not found for deletion: {}", id);
        }

        Ok(())
    }

    /// Count total AI prompts
    pub async fn count(&self) -> Result<usize, PersistenceError> {
        debug!("Counting total AI prompts");

        let conn = self.pool.get_connection()?;
        let mut stmt = conn.prepare("SELECT COUNT(*) FROM ai_prompts")?;
        let count: i64 = stmt.query_row([], |row| row.get(0))?;

        debug!("Total AI prompts: {}", count);
        Ok(count as usize)
    }
}

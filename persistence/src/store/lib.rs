//! Store struct and initialization
//! 
//! This file contains ONLY Store struct and initialization following Single Responsibility Principle.

use sqlx::SqlitePool;
use std::sync::Arc;
use crate::errors::PersistenceError;
use crate::logging::{log_db_operation_start, log_db_operation_success};

/// Main store struct for database operations
pub struct Store {
    pool: Arc<SqlitePool>,
}

impl Store {
    /// Create a new store instance
    pub async fn new(db_path: &str) -> Result<Self, PersistenceError> {
        log_db_operation_start("connect", "database");
        tracing::info!("Attempting to connect to database: {}", db_path);
        
        // Enable WAL mode for better concurrency
        let pool = SqlitePool::connect(&format!("sqlite:{}", db_path))
            .await
            .map_err(|e| {
                tracing::error!("Database connection failed: {}", e);
                PersistenceError::Database(e.to_string())
            })?;

        // Enable WAL mode
        sqlx::query("PRAGMA journal_mode=WAL")
            .execute(&pool)
            .await
            .map_err(|e| PersistenceError::Database(e.to_string()))?;

        // Create tables
        Self::create_tables(&pool).await?;

        log_db_operation_success("connect", "database", 0);
        
        Ok(Self {
            pool: Arc::new(pool),
        })
    }

    /// Create all required tables
    async fn create_tables(pool: &SqlitePool) -> Result<(), PersistenceError> {
        log_db_operation_start("create_tables", "all");
        
        let tables = [
            "CREATE TABLE IF NOT EXISTS workflows (id TEXT PRIMARY KEY, data JSON NOT NULL)",
            "CREATE TABLE IF NOT EXISTS workflow_executions (id TEXT PRIMARY KEY, data JSON NOT NULL)",
            "CREATE TABLE IF NOT EXISTS task_executions (id TEXT PRIMARY KEY, data JSON NOT NULL)",
            "CREATE TABLE IF NOT EXISTS user_prompts (id TEXT PRIMARY KEY, data JSON NOT NULL)",
            "CREATE TABLE IF NOT EXISTS audit_events (id TEXT PRIMARY KEY, data JSON NOT NULL)",
            "CREATE TABLE IF NOT EXISTS settings (id TEXT PRIMARY KEY, data JSON NOT NULL)",
        ];

        for table_sql in &tables {
            sqlx::query(table_sql)
                .execute(pool)
                .await
                .map_err(|e| PersistenceError::Database(e.to_string()))?;
        }

        log_db_operation_success("create_tables", "all", 0);
        Ok(())
    }

    /// Get the database pool (for internal use by store modules)
    pub(crate) fn pool(&self) -> &SqlitePool {
        &self.pool
    }
}

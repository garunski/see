//! Database connection pooling

use r2d2::{Pool, PooledConnection};
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::OpenFlags;
use std::sync::Arc;
use tracing::{debug, error, info};

use crate::error::PersistenceError;

/// Database connection pool for managing SQLite connections
pub struct DatabasePool {
    pool: Arc<Pool<SqliteConnectionManager>>,
    path: String,
}

impl DatabasePool {
    /// Create a new database pool
    pub fn new(path: &str, max_connections: u32) -> Result<Self, PersistenceError> {
        info!("Initializing database pool at: {}", path);

        let manager = SqliteConnectionManager::file(path)
            .with_flags(OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_CREATE)
            .with_init(|conn| {
                debug!("Initializing new database connection");

                // Set WAL mode for concurrent access
                conn.execute("PRAGMA journal_mode=WAL", [])?;
                debug!("Set journal mode to WAL");

                // Optimize for performance
                conn.execute("PRAGMA synchronous=NORMAL", [])?;
                conn.execute("PRAGMA cache_size=10000", [])?;
                conn.execute("PRAGMA temp_store=MEMORY", [])?;
                conn.execute("PRAGMA mmap_size=268435456", [])?; // 256MB

                // Enable foreign keys
                conn.execute("PRAGMA foreign_keys=ON", [])?;
                debug!("Enabled foreign key constraints");

                // Set busy timeout for concurrent access
                conn.busy_timeout(std::time::Duration::from_secs(30))?;
                debug!("Set busy timeout to 30 seconds");

                Ok(())
            });

        let pool = Pool::builder()
            .max_size(max_connections)
            .connection_timeout(std::time::Duration::from_secs(30))
            .idle_timeout(Some(std::time::Duration::from_secs(600))) // 10 minutes
            .build(manager)
            .map_err(|e| {
                error!("Failed to create connection pool: {}", e);
                PersistenceError::Connection(e.to_string())
            })?;

        info!(
            "Database pool created with {} max connections",
            max_connections
        );

        Ok(Self {
            pool: Arc::new(pool),
            path: path.to_string(),
        })
    }

    /// Get a connection from the pool
    pub fn get_connection(
        &self,
    ) -> Result<PooledConnection<SqliteConnectionManager>, PersistenceError> {
        debug!("Acquiring database connection from pool");
        self.pool.get().map_err(|e| {
            error!("Failed to get database connection: {}", e);
            PersistenceError::Connection(e.to_string())
        })
    }

    /// Get the current pool size
    pub fn pool_size(&self) -> u32 {
        self.pool.state().connections
    }

    /// Get the number of idle connections
    pub fn idle_connections(&self) -> u32 {
        self.pool.state().idle_connections
    }

    /// Get the database path
    pub fn path(&self) -> &str {
        &self.path
    }
}

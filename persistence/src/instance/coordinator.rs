//! Multi-instance coordination

use std::sync::Arc;
use tracing::{debug, info, instrument};

use crate::db::DatabasePool;
use crate::error::PersistenceError;

/// SQL query constants for coordination
const GET_ACTIVE_INSTANCES: &str = r#"
    SELECT DISTINCT instance_id 
    FROM workflow_executions 
    WHERE last_updated > datetime('now', '-5 minutes')
    AND instance_id IS NOT NULL
"#;

const CLEANUP_OLD_INSTANCES: &str = r#"
    DELETE FROM workflow_executions 
    WHERE last_updated < datetime('now', '-1 hour')
"#;

/// Coordinates multiple GUI instances
pub struct MultiInstanceCoordinator {
    pool: Arc<DatabasePool>,
}

/// Statistics about instances
#[derive(Debug, Clone)]
pub struct InstanceStats {
    pub total_workflows: i64,
    pub active_instances: usize,
    pub workflows_per_instance: Vec<(String, i64)>,
}

impl MultiInstanceCoordinator {
    pub fn new(pool: Arc<DatabasePool>) -> Self {
        info!("Created multi-instance coordinator");
        Self { pool }
    }

    #[instrument(skip(self))]
    pub async fn get_active_instances(&self) -> Result<Vec<String>, PersistenceError> {
        debug!("Getting active instances");

        let conn = self.pool.get_connection()?;
        let mut stmt = conn.prepare_cached(GET_ACTIVE_INSTANCES)?;

        let rows = stmt.query_map([], |row| row.get::<_, String>(0))?;

        let mut instances = Vec::new();
        for row in rows {
            instances.push(row?);
        }

        info!("Found {} active instances", instances.len());
        Ok(instances)
    }

    #[instrument(skip(self))]
    pub async fn cleanup_old_instances(&self) -> Result<usize, PersistenceError> {
        debug!("Cleaning up old instances");

        let conn = self.pool.get_connection()?;
        let changes = conn.execute(CLEANUP_OLD_INSTANCES, [])?;

        info!("Cleaned up {} old workflow records", changes);
        Ok(changes)
    }

    #[instrument(skip(self))]
    pub async fn get_instance_stats(&self) -> Result<InstanceStats, PersistenceError> {
        debug!("Getting instance statistics");

        let conn = self.pool.get_connection()?;

        // Get total workflows
        let total_workflows: i64 =
            conn.query_row("SELECT COUNT(*) FROM workflow_executions", [], |row| {
                row.get(0)
            })?;

        // Get active instances
        let active_instances = self.get_active_instances().await?;

        // Get workflows per instance
        let mut workflows_per_instance = Vec::new();
        for instance_id in &active_instances {
            let count: i64 = conn.query_row(
                "SELECT COUNT(*) FROM workflow_executions WHERE instance_id = ?1",
                [instance_id],
                |row| row.get(0),
            )?;
            workflows_per_instance.push((instance_id.clone(), count));
        }

        let stats = InstanceStats {
            total_workflows,
            active_instances: active_instances.len(),
            workflows_per_instance,
        };

        info!(
            "Instance stats: {} workflows, {} active instances",
            stats.total_workflows, stats.active_instances
        );

        Ok(stats)
    }
}

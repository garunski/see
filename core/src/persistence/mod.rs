pub mod models;
pub mod redb_store;

use async_trait::async_trait;
use uuid::Uuid;

use crate::errors::CoreError;

#[async_trait]
pub trait WorkflowStore: Send + Sync {
    async fn add_log(&self, id: Uuid, line: String) -> Result<(), CoreError>;
}

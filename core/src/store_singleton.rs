// Global store singleton ONLY

use persistence::Store;
use std::sync::{Arc, OnceLock};

static GLOBAL_STORE: OnceLock<Arc<Store>> = OnceLock::new();

/// Initialize the global persistence store singleton
pub async fn init_global_store() -> Result<(), String> {
    let db_path = get_database_path()?;
    let store = Store::new(&db_path)
        .await
        .map_err(|e| format!("Failed to create store: {}", e))?;

    GLOBAL_STORE
        .set(Arc::new(store))
        .map_err(|_| "Store already initialized".to_string())?;

    tracing::debug!("Global store initialized successfully");
    Ok(())
}

/// Get reference to the global persistence store
pub fn get_global_store() -> Result<Arc<Store>, String> {
    GLOBAL_STORE
        .get()
        .cloned()
        .ok_or_else(|| "Store not initialized. Call init_global_store() first.".to_string())
}

/// Get the database path for the store
fn get_database_path() -> Result<String, String> {
    // Use user home directory for database
    let home_dir =
        std::env::var("HOME").map_err(|_| "HOME environment variable not set".to_string())?;
    let data_dir = format!("{}/.s_e_e", home_dir);
    std::fs::create_dir_all(&data_dir)
        .map_err(|e| format!("Failed to create data directory: {}", e))?;
    let db_path = format!("{}/data.db", data_dir);
    tracing::debug!("Using database path: {}", db_path);
    Ok(db_path)
}

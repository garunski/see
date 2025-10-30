use s_e_e_persistence::Store;
use std::sync::{Arc, OnceLock};

static GLOBAL_STORE: OnceLock<Arc<Store>> = OnceLock::new();

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

pub fn get_global_store() -> Result<Arc<Store>, String> {
    GLOBAL_STORE
        .get()
        .cloned()
        .ok_or_else(|| "Store not initialized. Call init_global_store() first.".to_string())
}

fn get_database_path() -> Result<String, String> {
    let home_dir =
        std::env::var("HOME").map_err(|_| "HOME environment variable not set".to_string())?;
    let data_dir = format!("{}/.s_e_e", home_dir);
    std::fs::create_dir_all(&data_dir)
        .map_err(|e| format!("Failed to create data directory: {}", e))?;
    let db_path = format!("{}/data.db", data_dir);
    tracing::debug!("Using database path: {}", db_path);
    Ok(db_path)
}

fn get_test_database_path() -> Result<String, String> {
    let home_dir =
        std::env::var("HOME").map_err(|_| "HOME environment variable not set".to_string())?;
    let data_dir = format!("{}/.s_e_e", home_dir);
    std::fs::create_dir_all(&data_dir)
        .map_err(|e| format!("Failed to create data directory: {}", e))?;
    let db_path = format!("{}/test.db", data_dir);
    tracing::debug!("Using test database path: {}", db_path);
    Ok(db_path)
}

pub async fn init_test_store() -> Result<(), String> {
    // Check if store is already initialized
    if GLOBAL_STORE.get().is_some() {
        tracing::debug!("Test store already initialized, reusing existing store");
        return Ok(());
    }

    let db_path = get_test_database_path()?;

    if let Err(e) = std::fs::remove_file(&db_path) {
        tracing::debug!("Test database cleanup: {}", e);
    }

    let store = Store::new(&db_path)
        .await
        .map_err(|e| format!("Failed to create test store: {}", e))?;

    GLOBAL_STORE
        .set(Arc::new(store))
        .map_err(|_| "Store already initialized".to_string())?;

    tracing::debug!("Test store initialized at: {}", db_path);
    Ok(())
}

pub fn cleanup_test_db() -> Result<(), String> {
    let db_path = get_test_database_path()?;

    if let Err(e) = std::fs::remove_file(&db_path) {
        tracing::debug!("Test database cleanup: {}", e);
    }
    if let Err(e) = std::fs::remove_file(format!("{}-wal", db_path)) {
        tracing::debug!("WAL cleanup: {}", e);
    }
    if let Err(e) = std::fs::remove_file(format!("{}-shm", db_path)) {
        tracing::debug!("SHM cleanup: {}", e);
    }

    tracing::debug!("Test database cleanup attempted: {}", db_path);
    Ok(())
}

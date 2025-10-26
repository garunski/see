use tracing::{debug, info};

pub async fn clear_database() -> Result<(), String> {
    debug!("Clearing database");
    let store =
        s_e_e_core::get_global_store().map_err(|e| format!("Database unavailable: {}", e))?;

    store
        .clear_all_data()
        .await
        .map_err(|e| format!("Failed to clear database: {}", e))?;

    info!("Database cleared successfully");
    Ok(())
}

pub async fn clear_database() -> Result<(), String> {
    let store = see_core::get_global_store().map_err(|e| format!("Database unavailable: {}", e))?;

    store
        .clear_all_data()
        .await
        .map_err(|e| format!("Failed to clear database: {}", e))
}

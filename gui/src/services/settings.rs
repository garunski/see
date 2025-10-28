use s_e_e_core::AppSettings;

#[derive(Debug, thiserror::Error)]
pub enum SettingsError {
    #[error("Database not available: {0}")]
    DatabaseUnavailable(String),
    #[error("Failed to fetch settings: {0}")]
    FetchSettingsFailed(String),
    #[error("Failed to save settings: {0}")]
    SaveSettingsFailed(String),
}

pub struct SettingsService;

impl SettingsService {
    pub async fn fetch_settings() -> Result<AppSettings, SettingsError> {
        tracing::debug!("[SettingsService] fetch_settings: Getting global store");
        let store = s_e_e_core::get_global_store().map_err(|e| {
            tracing::error!("[SettingsService] Failed to get global store: {}", e);
            SettingsError::DatabaseUnavailable(e.to_string())
        })?;

        tracing::debug!("[SettingsService] fetch_settings: Loading settings from database");
        let result = store.load_settings().await;
        match &result {
            Ok(Some(settings)) => {
                tracing::info!(
                    "[SettingsService] Loaded settings from DB with theme: {:?}",
                    settings.theme
                );
                Ok(settings.clone())
            }
            Ok(None) => {
                tracing::info!("[SettingsService] No settings in DB, using defaults");
                Ok(AppSettings::default())
            }
            Err(e) => {
                tracing::error!("[SettingsService] DB error loading settings: {}", e);
                Err(SettingsError::FetchSettingsFailed(e.to_string()))
            }
        }
    }

    pub async fn save_settings(settings: AppSettings) -> Result<(), SettingsError> {
        tracing::info!(
            "[SettingsService] save_settings: Saving with theme: {:?}",
            settings.theme
        );
        let store = s_e_e_core::get_global_store().map_err(|e| {
            tracing::error!("[SettingsService] Failed to get global store: {}", e);
            SettingsError::DatabaseUnavailable(e.to_string())
        })?;

        tracing::debug!("[SettingsService] save_settings: Calling store.save_settings");
        let result = store.save_settings(&settings).await;
        match &result {
            Ok(_) => {
                tracing::info!("[SettingsService] Successfully saved settings to database");
            }
            Err(e) => {
                tracing::error!("[SettingsService] Failed to save settings: {}", e);
            }
        }
        result.map_err(|e| SettingsError::SaveSettingsFailed(e.to_string()))
    }
}

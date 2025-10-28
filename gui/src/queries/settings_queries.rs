use crate::services::SettingsService;
use dioxus_query::prelude::*;
use s_e_e_core::AppSettings;

#[derive(Clone, PartialEq, Hash, Eq)]
pub struct GetSettings;

impl QueryCapability for GetSettings {
    type Ok = AppSettings;
    type Err = String;
    type Keys = ();

    async fn run(&self, _: &Self::Keys) -> Result<Self::Ok, Self::Err> {
        tracing::debug!("[GetSettings] Fetching settings from service");
        let result = SettingsService::fetch_settings().await;
        match &result {
            Ok(settings) => {
                tracing::info!(
                    "[GetSettings] Successfully fetched settings with theme: {:?}",
                    settings.theme
                );
            }
            Err(e) => {
                tracing::error!("[GetSettings] Failed to fetch settings: {}", e);
            }
        }
        result.map_err(|e| e.to_string())
    }
}

#[derive(Clone, PartialEq, Hash, Eq)]
pub struct UpdateSettingsMutation;

impl MutationCapability for UpdateSettingsMutation {
    type Ok = ();
    type Err = String;
    type Keys = AppSettings;

    async fn run(&self, settings: &Self::Keys) -> Result<Self::Ok, Self::Err> {
        tracing::info!(
            "[UpdateSettingsMutation] Starting save with theme: {:?}",
            settings.theme
        );
        let result = SettingsService::save_settings(settings.clone()).await;
        match &result {
            Ok(_) => {
                tracing::info!("[UpdateSettingsMutation] Successfully saved settings to database");
            }
            Err(e) => {
                tracing::error!("[UpdateSettingsMutation] Failed to save settings: {}", e);
            }
        }
        result.map_err(|e| e.to_string())
    }

    async fn on_settled(&self, _: &Self::Keys, result: &Result<Self::Ok, Self::Err>) {
        tracing::debug!(
            "[UpdateSettingsMutation] on_settled called, result: {:?}",
            result
        );
        tracing::info!(
            "[UpdateSettingsMutation] Invalidating GetSettings cache to trigger refetch"
        );
        QueriesStorage::<GetSettings>::invalidate_matching(()).await;
        tracing::debug!("[UpdateSettingsMutation] Cache invalidated, GetSettings should refetch");
    }
}

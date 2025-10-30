use crate::services::SettingsService;
use dioxus::prelude::Signal;
use s_e_e_core::AppSettings;
use s_e_e_dioxus_query::prelude::*;
use std::rc::Rc;

pub fn use_settings_query() -> (QueryState<AppSettings>, impl Fn()) {
    let key = QueryKey::new(&["settings"]);

    let fetcher = move || async move {
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
    };

    let options = QueryOptions {
        stale_time: Some(60_000),
        cache_time: Some(300_000),
        ..Default::default()
    };

    use_query(key, fetcher, options)
}

pub fn use_update_settings_mutation() -> (Signal<MutationState<()>>, impl Fn(AppSettings)) {
    let mutation_fn = move |settings: AppSettings| async move {
        tracing::info!(
            "[UpdateSettingsMutation] Starting save with theme: {:?}",
            settings.theme
        );
        let result = SettingsService::save_settings(settings).await;
        match &result {
            Ok(_) => {
                tracing::info!("[UpdateSettingsMutation] Successfully saved settings to database");
            }
            Err(e) => {
                tracing::error!("[UpdateSettingsMutation] Failed to save settings: {}", e);
            }
        }
        result.map_err(|e| e.to_string())
    };

    let callbacks = MutationCallbacks {
        on_success: None,
        on_error: None,
        on_settled: Some(Rc::new(|| {
            tracing::info!("[UpdateSettingsMutation] Invalidating settings cache");
            invalidate_query(&QueryKey::new(&["settings"]));
        })),
        invalidate_keys: vec![QueryKey::new(&["settings"])],
        optimistic_update: None,
    };

    use_mutation(mutation_fn, callbacks)
}

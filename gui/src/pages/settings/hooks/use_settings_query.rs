use crate::queries::use_settings_query as use_settings_query_hook;
use s_e_e_core::AppSettings;

pub fn use_settings_query() -> Result<AppSettings, String> {
    let (state, _refetch) = use_settings_query_hook();

    if state.is_loading {
        Err("Loading settings...".to_string())
    } else if state.is_error {
        Err(state
            .error
            .clone()
            .unwrap_or_else(|| "Failed to load settings".to_string()))
    } else {
        Ok(state.data.clone().unwrap_or_default())
    }
}

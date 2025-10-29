use crate::queries::use_settings_query;
use dioxus::prelude::*;
use s_e_e_core::Theme;

pub fn use_theme() -> Memo<Theme> {
    tracing::trace!("[use_theme] Called, setting up query");
    let (settings_state, _refetch) = use_settings_query();

    use_memo(move || {
        let theme = if settings_state.is_loading {
            Theme::System
        } else if settings_state.is_error {
            tracing::error!("[use_theme] Query failed, using System as fallback");
            Theme::System
        } else if let Some(settings) = &settings_state.data {
            tracing::debug!(
                "[use_theme] Query returned settings with theme: {:?}",
                settings.theme
            );
            settings.theme.clone()
        } else {
            Theme::System
        };
        tracing::trace!("[use_theme] Returning theme: {:?}", theme);
        theme
    })
}

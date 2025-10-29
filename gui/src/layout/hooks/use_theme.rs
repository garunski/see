use crate::queries::GetSettings;
use dioxus::prelude::*;
use dioxus_query::prelude::*;
use s_e_e_core::Theme;

pub fn use_theme() -> Memo<Theme> {
    tracing::trace!("[use_theme] Called, setting up query");
    let query = use_query(Query::new((), GetSettings));

    use_memo(move || {
        let query_result = query
            .suspend()
            .unwrap_or_else(|_| Ok(s_e_e_core::AppSettings::default()));
        let theme = match query_result {
            Ok(settings) => {
                tracing::debug!(
                    "[use_theme] Query returned settings with theme: {:?}",
                    settings.theme
                );
                settings.theme.clone()
            }
            Err(e) => {
                tracing::error!("[use_theme] Query failed: {}, using System as fallback", e);
                Theme::System
            }
        };
        tracing::trace!("[use_theme] Returning theme: {:?}", theme);
        theme
    })
}

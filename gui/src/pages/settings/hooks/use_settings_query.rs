use crate::queries::GetSettings;
use dioxus_query::prelude::*;
use s_e_e_core::AppSettings;

pub fn use_settings_query() -> Result<AppSettings, String> {
    let query_result = use_query(Query::new((), GetSettings))
        .suspend()
        .map_err(|_| String::from("Failed to initialize query"))?;

    match query_result {
        Ok(value) => Ok(value),
        Err(e) => {
            tracing::error!("Failed to load settings: {}", e);
            Err(format!("Failed to load settings: {}", e))
        }
    }
}

use crate::queries::GetPrompts;
use dioxus_query::prelude::*;
use s_e_e_core::Prompt;

pub fn use_prompts_list() -> Result<Vec<Prompt>, String> {
    let query_result = use_query(Query::new((), GetPrompts))
        .suspend()
        .map_err(|_| String::from("Failed to initialize query"))?;
    match query_result {
        Ok(value) => Ok(value),
        Err(e) => {
            tracing::error!("Failed to load prompts: {}", e);
            Err(format!("Failed to load prompts: {}", e))
        }
    }
}

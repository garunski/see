use crate::queries::prompt_queries::use_prompts_query;
use s_e_e_core::Prompt;

pub fn use_prompts_list() -> Result<Vec<Prompt>, String> {
    let (state, _refetch) = use_prompts_query();

    if state.is_loading {
        // Still loading initial data
        return Err("Loading prompts...".to_string());
    }

    if state.is_error {
        let error_msg = state.error.unwrap_or_else(|| "Unknown error".to_string());
        tracing::error!("Failed to load prompts: {}", error_msg);
        return Err(format!("Failed to load prompts: {}", error_msg));
    }

    match state.data {
        Some(prompts) => Ok(prompts),
        None => Err("No data available".to_string()),
    }
}

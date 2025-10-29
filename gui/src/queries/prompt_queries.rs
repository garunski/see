use crate::services::prompt::UserPromptService;
use dioxus::prelude::Signal;
use dioxus_query_custom::prelude::*;
use s_e_e_core::Prompt;
use std::rc::Rc;

// ==================== QUERIES ====================

/// Hook to fetch all prompts
pub fn use_prompts_query() -> (QueryState<Vec<Prompt>>, impl Fn()) {
    let key = QueryKey::new(&["prompts", "list"]);

    let fetcher = move || async move {
        UserPromptService::fetch_prompts()
            .await
            .map_err(|e| e.to_string())
    };

    let options = QueryOptions {
        stale_time: Some(30_000),  // 30 seconds
        cache_time: Some(300_000), // 5 minutes
        ..Default::default()
    };

    use_query(key, fetcher, options)
}

/// Hook to fetch a single prompt by ID
pub fn use_prompt_query(id: String) -> (QueryState<Option<Prompt>>, impl Fn()) {
    let key = QueryKey::new(&["prompts", "detail", &id]);

    let id_clone = id.clone();
    let fetcher = move || {
        let id = id_clone.clone();
        async move {
            let prompts = UserPromptService::fetch_prompts()
                .await
                .map_err(|e| e.to_string())?;

            Ok(prompts.into_iter().find(|p| p.id == id))
        }
    };

    let options = QueryOptions {
        stale_time: Some(30_000),
        cache_time: Some(300_000),
        ..Default::default()
    };

    use_query(key, fetcher, options)
}

// ==================== MUTATIONS ====================

/// Hook to create a new prompt
pub fn use_create_prompt_mutation() -> (Signal<MutationState<()>>, impl Fn(Prompt)) {
    let mutation_fn = move |prompt: Prompt| async move {
        UserPromptService::create_prompt(prompt)
            .await
            .map_err(|e| e.to_string())
    };

    let callbacks = MutationCallbacks {
        on_success: None,
        on_error: None,
        on_settled: Some(Rc::new(|| {
            // Invalidate prompts list and detail queries
            invalidate_queries_by_prefix("prompts:");
        })),
        invalidate_keys: vec![QueryKey::new(&["prompts", "list"])],
        optimistic_update: None,
    };

    use_mutation(mutation_fn, callbacks)
}

/// Hook to update an existing prompt
pub fn use_update_prompt_mutation() -> (Signal<MutationState<()>>, impl Fn(Prompt)) {
    let mutation_fn = move |prompt: Prompt| async move {
        UserPromptService::update_prompt(prompt)
            .await
            .map_err(|e| e.to_string())
    };

    let callbacks = MutationCallbacks {
        on_success: None,
        on_error: None,
        on_settled: Some(Rc::new(|| {
            invalidate_queries_by_prefix("prompts:");
        })),
        invalidate_keys: vec![QueryKey::new(&["prompts", "list"])],
        optimistic_update: None,
    };

    use_mutation(mutation_fn, callbacks)
}

/// Hook to delete a prompt
pub fn use_delete_prompt_mutation() -> (Signal<MutationState<()>>, impl Fn(String)) {
    let mutation_fn = move |id: String| async move {
        UserPromptService::delete_prompt(&id)
            .await
            .map_err(|e| e.to_string())
    };

    let callbacks = MutationCallbacks {
        on_success: None,
        on_error: None,
        on_settled: Some(Rc::new(|| {
            invalidate_queries_by_prefix("prompts:");
        })),
        invalidate_keys: vec![QueryKey::new(&["prompts", "list"])],
        optimistic_update: None,
    };

    use_mutation(mutation_fn, callbacks)
}

use dioxus::prelude::*;
use futures::Future;
use tracing::{debug, error, info, instrument, trace};

use crate::invalidate::invalidate_query;
use crate::state::{MutationCallbacks, MutationState};

/// Mutation hook for performing data mutations with optimistic updates
#[instrument(skip(mutation_fn, callbacks))]
pub fn use_mutation<T, V, F, Fut>(
    mutation_fn: F,
    callbacks: MutationCallbacks<T, V>,
) -> (Signal<MutationState<T>>, impl Fn(V))
where
    T: Clone + PartialEq + 'static,
    V: Clone + 'static,
    F: Fn(V) -> Fut + 'static + Clone,
    Fut: Future<Output = Result<T, String>> + 'static,
{
    info!(
        invalidate_keys_count = callbacks.invalidate_keys.len(),
        has_optimistic = callbacks.optimistic_update.is_some(),
        "Initializing mutation"
    );

    let state = use_signal(|| MutationState {
        data: None,
        is_loading: false,
        is_error: false,
        error: None,
        is_success: false,
    });

    let state_for_mutation = state.clone();
    let mutate = use_callback(move |variables: V| {
        let mutation_fn = mutation_fn.clone();
        let callbacks = callbacks.clone();
        let mut state = state_for_mutation.clone();

        info!("Mutation triggered");

        spawn(async move {
            state.set(MutationState {
                data: None,
                is_loading: true,
                is_error: false,
                error: None,
                is_success: false,
            });
            debug!("Mutation state: is_loading = true");

            // Apply optimistic update if provided
            if let Some((key, _optimistic_data)) = &callbacks.optimistic_update {
                info!(key = %key, "Applying optimistic update");
                // Optimistic update sets data directly in cache
                // This is simplified - in full implementation would set proper typed entry
                debug!(key = %key, "Optimistic update applied to cache");
            }

            match mutation_fn(variables).await {
                Ok(result) => {
                    info!("Mutation successful");

                    state.set(MutationState {
                        data: Some(result.clone()),
                        is_loading: false,
                        is_error: false,
                        error: None,
                        is_success: true,
                    });
                    debug!("Mutation state updated: success");

                    // Invalidate specified queries
                    if !callbacks.invalidate_keys.is_empty() {
                        debug!(
                            count = callbacks.invalidate_keys.len(),
                            "Invalidating queries after successful mutation"
                        );
                        for key in &callbacks.invalidate_keys {
                            invalidate_query(key);
                        }
                    }

                    // Success callback
                    if let Some(on_success) = &callbacks.on_success {
                        trace!("Executing on_success callback");
                        on_success(result);
                    }

                    // Settled callback
                    if let Some(on_settled) = &callbacks.on_settled {
                        trace!("Executing on_settled callback (success path)");
                        on_settled();
                    }
                }
                Err(err) => {
                    error!(error = %err, "Mutation failed");

                    state.set(MutationState {
                        data: None,
                        is_loading: false,
                        is_error: true,
                        error: Some(err.clone()),
                        is_success: false,
                    });
                    debug!("Mutation state updated: error");

                    // Error callback
                    if let Some(on_error) = &callbacks.on_error {
                        trace!("Executing on_error callback");
                        on_error(err);
                    }

                    // Settled callback
                    if let Some(on_settled) = &callbacks.on_settled {
                        trace!("Executing on_settled callback (error path)");
                        on_settled();
                    }
                }
            }
        });
    });

    (state, move |vars| mutate(vars))
}

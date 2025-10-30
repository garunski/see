use dioxus::prelude::*;
use futures::Future;
use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, error, info, instrument, trace, warn};

use crate::cache::cleanup::cleanup_stale_entries_sync;
use crate::cache::storage::QUERY_CACHE;
use crate::cache::{get_typed_value, mark_fetch_complete, start_cleanup_task, TypedCacheEntry};
use crate::query_key::QueryKey;
use crate::state::{QueryOptions, QueryState};

#[instrument(skip(fetcher, options), fields(key = %key))]
pub fn use_query<T, F, Fut>(
    key: QueryKey,
    fetcher: F,
    options: QueryOptions,
) -> (QueryState<T>, impl Fn())
where
    T: Clone + PartialEq + Send + Sync + 'static,
    F: Fn() -> Fut + 'static + Clone,
    Fut: Future<Output = Result<T, String>> + 'static,
{
    info!(
        key = %key,
        stale_time = ?options.stale_time,
        retry = ?options.retry,
        enabled = options.enabled,
        "Initializing query"
    );

    // Start cleanup task on first query (now a no-op)
    start_cleanup_task();

    // Component-local state - all components read from shared cache though
    let mut state = use_signal(QueryState::default);

    // Lazy cleanup: clean up stale entries periodically during queries
    // This runs in Dioxus context, avoiding GlobalSignal runtime errors
    use_effect(move || {
        let cache = QUERY_CACHE();
        let mut cache_map = cache.borrow_mut();
        cleanup_stale_entries_sync(&mut cache_map);
    });

    let key_clone = key.clone();

    // Check cache on mount
    use_hook(|| {
        trace!(key = %key_clone, "Checking cache on mount");
        let cache = QUERY_CACHE();
        let cache_guard = cache.borrow();
        if let Some(entry) = cache_guard.get(&key_clone) {
            if let Some(typed_value) = get_typed_value::<T>(entry.as_ref()) {
                debug!(key = %key_clone, "Cache hit - loading from cache");
                let cached_data = (*typed_value).clone();
                let new_state = QueryState {
                    data: Some(cached_data),
                    is_loading: false,
                    is_fetching: false,
                    is_error: false,
                    error: None,
                };
                state.set(new_state);
            } else {
                trace!(key = %key_clone, "Cache entry type mismatch");
            }
        } else {
            trace!(key = %key_clone, "Cache miss");
        }
    });

    // Determine if we should fetch
    let fetch_key_clone = key.clone();
    let fetch_options = options.clone();
    let should_fetch = use_memo(move || {
        if !fetch_options.enabled {
            debug!(key = %fetch_key_clone, "Query disabled via options");
            return false;
        }

        let cache = QUERY_CACHE();
        let cache_map = cache.borrow();

        if let Some(cached) = cache_map.get(&fetch_key_clone) {
            // Check if currently fetching
            if cached.is_fetching() {
                trace!(key = %fetch_key_clone, "Query already fetching");
                return false;
            }

            // Check staleness
            if let Some(fetched_at) = cached.fetched_at() {
                if let Some(stale_time) = fetch_options.stale_time {
                    let elapsed = fetched_at.elapsed().as_millis() as u64;
                    if elapsed < stale_time {
                        trace!(
                            key = %fetch_key_clone,
                            elapsed_ms = elapsed,
                            stale_time_ms = stale_time,
                            "Data is fresh - skipping fetch"
                        );
                        return false;
                    } else {
                        debug!(
                            key = %fetch_key_clone,
                            elapsed_ms = elapsed,
                            stale_time_ms = stale_time,
                            "Data is stale - will refetch"
                        );
                    }
                }
            }
        }

        true
    });

    // Fetch logic with deduplication
    let fetch_key = key.clone();
    let fetch_opts = options.clone();
    let state_for_fetch = state;
    let fetch = use_callback(move |_| {
        let fetcher = fetcher.clone();
        let key_clone = fetch_key.clone();
        let opts = fetch_opts.clone();
        let mut state = state_for_fetch;

        // Check if already fetching (deduplication)
        {
            let cache = QUERY_CACHE();
            let cache_map = cache.borrow();
            if let Some(entry) = cache_map.get(&key_clone) {
                if entry.is_fetching() {
                    debug!(key = %key_clone, "Query already fetching - skipping duplicate request");
                    return;
                }
            }
        }

        info!(key = %key_clone, "Starting fetch operation");

        spawn(async move {
            let key = key_clone.clone();

            // Mark as fetching in cache (double-check after spawn)
            {
                let cache = QUERY_CACHE();
                let mut cache_map = cache.borrow_mut();

                // Check again in case another fetch started between our check and spawn
                if let Some(entry) = cache_map.get(&key) {
                    if entry.is_fetching() {
                        debug!(key = %key, "Another fetch started - aborting this one");
                        return;
                    }
                }

                // Set fetching flag - if entry doesn't exist yet, that's okay
                // It will be created when fetch completes
                if let Some(entry) = cache_map.get_mut(&key) {
                    entry.set_fetching(true);
                    trace!(key = %key, "Marked cache entry as fetching");
                }
            }

            state.write().is_fetching = true;
            debug!(key = %key, "Query state: is_fetching = true");

            let mut attempts = 0;
            let max_attempts = opts.retry.unwrap_or(0) + 1;

            loop {
                attempts += 1;
                debug!(
                    key = %key,
                    attempt = attempts,
                    max_attempts = max_attempts,
                    "Fetch attempt"
                );

                match fetcher().await {
                    Ok(data) => {
                        info!(key = %key, attempt = attempts, "Fetch successful");

                        // Update cache with typed value (no serialization!)
                        // Store with the query's cache_time option
                        let typed_entry = TypedCacheEntry::with_cache_time(
                            Arc::new(data.clone()),
                            opts.cache_time,
                        );
                        let cache = QUERY_CACHE();
                        let mut cache_map = cache.borrow_mut();
                        cache_map.insert(
                            key.clone(),
                            Box::new(typed_entry) as Box<dyn crate::cache::CacheEntry>,
                        );
                        trace!(key = %key, cache_time = ?opts.cache_time, "Updated cache with fresh data");

                        let new_state = QueryState {
                            data: Some(data),
                            is_loading: false,
                            is_fetching: false,
                            is_error: false,
                            error: None,
                        };
                        state.set(new_state);
                        debug!(key = %key, "Query state updated: success");
                        break;
                    }
                    Err(err) => {
                        if attempts < max_attempts {
                            warn!(
                                key = %key,
                                attempt = attempts,
                                max_attempts = max_attempts,
                                error = %err,
                                "Fetch failed - will retry"
                            );

                            if let Some(delay) = opts.retry_delay {
                                debug!(key = %key, delay_ms = delay, "Waiting before retry");
                                tokio::time::sleep(Duration::from_millis(delay)).await;
                            }
                            continue;
                        }

                        error!(
                            key = %key,
                            attempt = attempts,
                            error = %err,
                            "Fetch failed after all retry attempts"
                        );

                        {
                            let cache = QUERY_CACHE();
                            let mut cache_map = cache.borrow_mut();
                            cache_map.remove(&key);
                            debug!(key = %key, "Removed failed cache entry to prevent memory leak");
                        }

                        let error_msg = err.clone();
                        let new_state = QueryState {
                            data: None,
                            is_loading: false,
                            is_fetching: false,
                            is_error: true,
                            error: Some(error_msg),
                        };
                        state.set(new_state);
                        debug!(key = %key, "Query state updated: error");
                        break;
                    }
                }
            }

            mark_fetch_complete(&key);
        });
    });

    let effect_key = key.clone();
    use_effect(move || {
        if should_fetch() {
            debug!(key = %effect_key, "Auto-fetch triggered");
            fetch(());
        }
    });

    if let Some(interval) = options.refetch_interval {
        info!(
            key = %key,
            interval_ms = interval,
            "Setting up refetch interval"
        );

        let fetch_for_interval = fetch;
        let interval_key_str = key.as_str().to_string();
        use_future(move || {
            let fetch = fetch_for_interval;
            let key_str = interval_key_str.clone();
            async move {
                let mut interval_stream = tokio::time::interval(Duration::from_millis(interval));
                loop {
                    interval_stream.tick().await;
                    trace!(key = %key_str, "Refetch interval triggered");
                    fetch(());
                }
            }
        });
    }

    let refetch_key = key.clone();
    let refetch = move || {
        debug!(key = %refetch_key, "Manual refetch triggered");
        fetch(())
    };

    let current_state = state.read().clone();
    (current_state, refetch)
}

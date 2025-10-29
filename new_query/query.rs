use std::{
    cell::RefCell,
    collections::HashMap,
    hash::{Hash, Hasher},
    rc::Rc,
    time::{Duration, Instant},
};
use dioxus::prelude::*;
use futures::Future;
use serde::{de::DeserializeOwned, Serialize};
use tracing::{debug, error, info, trace, warn, instrument};

// ---------- QUERY KEY ----------

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct QueryKey(String);

impl QueryKey {
    pub fn new(parts: &[&str]) -> Self {
        let key = parts.join(":");
        trace!(key = %key, "Creating new QueryKey");
        Self(key)
    }
    
    pub fn from_string(s: String) -> Self {
        trace!(key = %s, "Creating QueryKey from string");
        Self(s)
    }
}

impl std::fmt::Display for QueryKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

// ---------- INTERNAL CACHE ----------

static QUERY_CACHE: GlobalSignal<Rc<RefCell<HashMap<QueryKey, CachedQuery>>>> = 
    Signal::global(|| {
        debug!("Initializing global query cache");
        Rc::new(RefCell::new(HashMap::new()))
    });

#[derive(Clone)]
struct CachedQuery {
    value: Option<Rc<serde_json::Value>>,
    fetched_at: Option<Instant>,
    is_fetching: bool,
}

// ---------- CONFIG & STATE ----------

#[derive(Clone, Debug)]
pub struct QueryOptions {
    pub stale_time: Option<u64>,           // ms
    pub cache_time: Option<u64>,           // ms - how long to keep unused cache
    pub refetch_interval: Option<u64>,     // ms
    pub retry: Option<u8>,                 // retry count
    pub retry_delay: Option<u64>,          // ms between retries
    pub refetch_on_mount: bool,
    pub refetch_on_window_focus: bool,
    pub enabled: bool,                     // can disable query
}

impl Default for QueryOptions {
    fn default() -> Self {
        Self {
            stale_time: Some(0),
            cache_time: Some(300_000), // 5 minutes
            refetch_interval: None,
            retry: Some(3),
            retry_delay: Some(1000),
            refetch_on_mount: true,
            refetch_on_window_focus: false,
            enabled: true,
        }
    }
}

#[derive(Clone, PartialEq)]
pub struct QueryState<T: Clone + PartialEq> {
    pub data: Option<T>,
    pub is_loading: bool,
    pub is_fetching: bool,
    pub is_error: bool,
    pub error: Option<String>,
}

// ---------- USE_QUERY ----------

#[instrument(skip(fetcher, options), fields(key = %key))]
pub fn use_query<T, F, Fut>(
    key: QueryKey,
    fetcher: F,
    options: QueryOptions,
) -> (QueryState<T>, impl Fn())
where
    T: Clone + DeserializeOwned + Serialize + PartialEq + 'static,
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

    let mut state = use_signal(|| QueryState {
        data: None,
        is_loading: true,
        is_fetching: false,
        is_error: false,
        error: None,
    });

    let cache = QUERY_CACHE;
    let key_clone = key.clone();
    
    // Check cache on mount
    use_hook(|| {
        trace!(key = %key_clone, "Checking cache on mount");
        if let Some(cached) = cache.read().borrow().get(&key_clone) {
            if let Some(value) = &cached.value {
                if let Ok(parsed) = serde_json::from_value::<T>((**value).clone()) {
                    debug!(key = %key_clone, "Cache hit - loading from cache");
                    state.set(QueryState {
                        data: Some(parsed),
                        is_loading: false,
                        is_fetching: false,
                        is_error: false,
                        error: None,
                    });
                } else {
                    warn!(key = %key_clone, "Cache hit but failed to deserialize");
                }
            }
        } else {
            trace!(key = %key_clone, "Cache miss");
        }
    });

    // Determine if we should fetch
    let should_fetch = use_memo(move || {
        if !options.enabled {
            debug!(key = %key, "Query disabled via options");
            return false;
        }

        let cache_guard = cache.read();
        let cache_map = cache_guard.borrow();
        
        if let Some(cached) = cache_map.get(&key) {
            // Check if currently fetching
            if cached.is_fetching {
                trace!(key = %key, "Query already fetching");
                return false;
            }
            
            // Check staleness
            if let Some(fetched_at) = cached.fetched_at {
                if let Some(stale_time) = options.stale_time {
                    let elapsed = fetched_at.elapsed().as_millis();
                    if elapsed < stale_time as u128 {
                        trace!(
                            key = %key,
                            elapsed_ms = elapsed,
                            stale_time_ms = stale_time,
                            "Data is fresh - skipping fetch"
                        );
                        return false;
                    } else {
                        debug!(
                            key = %key,
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

    // Fetch logic
    let fetch_key = key.clone();
    let fetch_opts = options.clone();
    let fetch = use_callback(move |_| {
        let fetcher = fetcher.clone();
        let key = fetch_key.clone();
        let opts = fetch_opts.clone();
        
        info!(key = %key, "Starting fetch operation");
        
        spawn(async move {
            // Mark as fetching
            {
                let mut cache_map = cache.write().borrow_mut();
                if let Some(cached) = cache_map.get_mut(&key) {
                    cached.is_fetching = true;
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
                        
                        // Update cache
                        if let Ok(json) = serde_json::to_value(&data) {
                            let mut cache_map = cache.write().borrow_mut();
                            cache_map.insert(
                                key.clone(),
                                CachedQuery {
                                    value: Some(Rc::new(json)),
                                    fetched_at: Some(Instant::now()),
                                    is_fetching: false,
                                },
                            );
                            trace!(key = %key, "Updated cache with fresh data");
                        } else {
                            warn!(key = %key, "Failed to serialize data to JSON for cache");
                        }
                        
                        // Update state
                        state.set(QueryState {
                            data: Some(data),
                            is_loading: false,
                            is_fetching: false,
                            is_error: false,
                            error: None,
                        });
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
                        
                        // Mark fetch complete in cache
                        {
                            let mut cache_map = cache.write().borrow_mut();
                            if let Some(cached) = cache_map.get_mut(&key) {
                                cached.is_fetching = false;
                                trace!(key = %key, "Marked cache entry as not fetching");
                            }
                        }
                        
                        state.set(QueryState {
                            data: None,
                            is_loading: false,
                            is_fetching: false,
                            is_error: true,
                            error: Some(err.clone()),
                        });
                        debug!(key = %key, "Query state updated: error");
                        break;
                    }
                }
            }
        });
    });

    // Auto-fetch on mount or when dependencies change
    use_effect(move || {
        if should_fetch() {
            debug!(key = %key, "Auto-fetch triggered");
            fetch(());
        }
    });

    // Refetch interval
    if let Some(interval) = options.refetch_interval {
        info!(
            key = %key,
            interval_ms = interval,
            "Setting up refetch interval"
        );
        
        use_future(move || {
            let fetch = fetch.clone();
            async move {
                loop {
                    tokio::time::sleep(Duration::from_millis(interval)).await;
                    trace!(key = %key, "Refetch interval triggered");
                    fetch(());
                }
            }
        });
    }

    let refetch = move || {
        debug!(key = %key, "Manual refetch triggered");
        fetch(())
    };
    
    (state.read().clone(), refetch)
}

// ---------- INVALIDATION ----------

#[instrument]
pub fn invalidate_query(key: &QueryKey) {
    info!(key = %key, "Invalidating query");
    let mut cache_map = QUERY_CACHE.write().borrow_mut();
    if cache_map.remove(key).is_some() {
        debug!(key = %key, "Query removed from cache");
    } else {
        trace!(key = %key, "Query not found in cache");
    }
}

#[instrument]
pub fn invalidate_queries_by_prefix(prefix: &str) {
    info!(prefix = prefix, "Invalidating queries by prefix");
    let mut cache_map = QUERY_CACHE.write().borrow_mut();
    let before_count = cache_map.len();
    cache_map.retain(|k, _| !k.0.starts_with(prefix));
    let removed_count = before_count - cache_map.len();
    debug!(
        prefix = prefix,
        removed_count = removed_count,
        "Queries invalidated by prefix"
    );
}

#[instrument]
pub fn invalidate_all_queries() {
    info!("Invalidating all queries");
    let mut cache_map = QUERY_CACHE.write().borrow_mut();
    let count = cache_map.len();
    cache_map.clear();
    debug!(cleared_count = count, "All queries invalidated");
}

#[instrument(skip(data))]
pub fn set_query_data<T>(key: &QueryKey, data: T)
where
    T: Serialize + 'static,
{
    info!(key = %key, "Setting query data manually");
    if let Ok(json) = serde_json::to_value(&data) {
        let mut cache_map = QUERY_CACHE.write().borrow_mut();
        cache_map.insert(
            key.clone(),
            CachedQuery {
                value: Some(Rc::new(json)),
                fetched_at: Some(Instant::now()),
                is_fetching: false,
            },
        );
        debug!(key = %key, "Query data set in cache");
    } else {
        error!(key = %key, "Failed to serialize data for manual cache set");
    }
}

// ---------- USE_MUTATION ----------

#[derive(Clone, PartialEq)]
pub struct MutationState<T: Clone + PartialEq> {
    pub data: Option<T>,
    pub is_loading: bool,
    pub is_error: bool,
    pub error: Option<String>,
    pub is_success: bool,
}

pub struct MutationCallbacks<T, V> {
    pub on_success: Option<Rc<dyn Fn(T)>>,
    pub on_error: Option<Rc<dyn Fn(String)>>,
    pub on_settled: Option<Rc<dyn Fn()>>,
    pub invalidate_keys: Vec<QueryKey>,
    pub optimistic_update: Option<(QueryKey, V)>,
}

impl<T, V> Default for MutationCallbacks<T, V> {
    fn default() -> Self {
        Self {
            on_success: None,
            on_error: None,
            on_settled: None,
            invalidate_keys: Vec::new(),
            optimistic_update: None,
        }
    }
}

#[instrument(skip(mutation_fn, callbacks))]
pub fn use_mutation<T, V, F, Fut>(
    mutation_fn: F,
    callbacks: MutationCallbacks<T, V>,
) -> (MutationState<T>, impl Fn(V))
where
    T: Clone + DeserializeOwned + Serialize + PartialEq + 'static,
    V: Clone + Serialize + 'static,
    F: Fn(V) -> Fut + 'static + Clone,
    Fut: Future<Output = Result<T, String>> + 'static,
{
    info!(
        invalidate_keys_count = callbacks.invalidate_keys.len(),
        has_optimistic = callbacks.optimistic_update.is_some(),
        "Initializing mutation"
    );

    let mut state = use_signal(|| MutationState {
        data: None,
        is_loading: false,
        is_error: false,
        error: None,
        is_success: false,
    });

    let mutate = use_callback(move |variables: V| {
        let mutation_fn = mutation_fn.clone();
        let callbacks = callbacks.clone();
        
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

            // Optimistic update
            if let Some((key, optimistic_data)) = &callbacks.optimistic_update {
                info!(key = %key, "Applying optimistic update");
                set_query_data(key, optimistic_data.clone());
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

    (state.read().clone(), move |vars| mutate(vars))
}

// ---------- HELPER: USE_QUERIES (parallel) ----------

#[instrument(skip(queries))]
pub fn use_queries<T, F, Fut>(
    queries: Vec<(QueryKey, F, QueryOptions)>,
) -> Vec<QueryState<T>>
where
    T: Clone + DeserializeOwned + Serialize + PartialEq + 'static,
    F: Fn() -> Fut + 'static + Clone,
    Fut: Future<Output = Result<T, String>> + 'static,
{
    info!(query_count = queries.len(), "Initializing parallel queries");
    let mut results = Vec::new();
    
    for (key, fetcher, opts) in queries {
        trace!(key = %key, "Adding query to parallel batch");
        let (state, _) = use_query(key, fetcher, opts);
        results.push(state);
    }
    
    debug!(query_count = results.len(), "Parallel queries initialized");
    results
}

// ---------- CACHE INSPECTION (for debugging) ----------

#[instrument]
pub fn get_cache_stats() -> (usize, Vec<String>) {
    let cache_map = QUERY_CACHE.read().borrow();
    let size = cache_map.len();
    let keys: Vec<String> = cache_map.keys().map(|k| k.0.clone()).collect();
    
    debug!(
        cache_size = size,
        keys = ?keys,
        "Cache statistics retrieved"
    );
    
    (size, keys)
}
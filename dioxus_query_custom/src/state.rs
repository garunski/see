use std::rc::Rc;

/// Query state returned by `use_query`
#[derive(Clone, PartialEq)]
pub struct QueryState<T: Clone + PartialEq> {
    pub data: Option<T>,
    pub is_loading: bool,
    pub is_fetching: bool,
    pub is_error: bool,
    pub error: Option<String>,
}

impl<T: Clone + PartialEq> Default for QueryState<T> {
    fn default() -> Self {
        Self {
            data: None,
            is_loading: true,
            is_fetching: false,
            is_error: false,
            error: None,
        }
    }
}

/// Mutation state returned by `use_mutation`
#[derive(Clone, PartialEq)]
pub struct MutationState<T: Clone + PartialEq> {
    pub data: Option<T>,
    pub is_loading: bool,
    pub is_error: bool,
    pub error: Option<String>,
    pub is_success: bool,
}

impl<T: Clone + PartialEq> Default for MutationState<T> {
    fn default() -> Self {
        Self {
            data: None,
            is_loading: false,
            is_error: false,
            error: None,
            is_success: false,
        }
    }
}

/// Options for configuring query behavior
#[derive(Clone, Debug)]
pub struct QueryOptions {
    /// Time in milliseconds before data is considered stale
    pub stale_time: Option<u64>,
    /// Time in milliseconds before cached data is evicted
    pub cache_time: Option<u64>,
    /// Interval in milliseconds for automatic refetching
    pub refetch_interval: Option<u64>,
    /// Number of retry attempts on failure
    pub retry: Option<u8>,
    /// Delay in milliseconds between retry attempts
    pub retry_delay: Option<u64>,
    /// Whether to refetch when component mounts
    pub refetch_on_mount: bool,
    /// Whether to refetch when window regains focus
    pub refetch_on_window_focus: bool,
    /// Whether the query is enabled
    pub enabled: bool,
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

/// Callbacks for mutation lifecycle events
pub struct MutationCallbacks<T, V> {
    /// Called when mutation succeeds
    pub on_success: Option<Rc<dyn Fn(T)>>,
    /// Called when mutation fails
    pub on_error: Option<Rc<dyn Fn(String)>>,
    /// Called when mutation completes (success or error)
    pub on_settled: Option<Rc<dyn Fn()>>,
    /// Query keys to invalidate after successful mutation
    pub invalidate_keys: Vec<crate::query_key::QueryKey>,
    /// Optimistic update (key, value) pair
    pub optimistic_update: Option<(crate::query_key::QueryKey, V)>,
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

impl<T, V: Clone> Clone for MutationCallbacks<T, V> {
    fn clone(&self) -> Self {
        Self {
            on_success: self.on_success.clone(),
            on_error: self.on_error.clone(),
            on_settled: self.on_settled.clone(),
            invalidate_keys: self.invalidate_keys.clone(),
            optimistic_update: self.optimistic_update.clone(),
        }
    }
}

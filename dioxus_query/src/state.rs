use std::rc::Rc;

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

#[derive(Clone, Debug)]
pub struct QueryOptions {
    pub stale_time: Option<u64>,

    pub cache_time: Option<u64>,

    pub refetch_interval: Option<u64>,

    pub retry: Option<u8>,

    pub retry_delay: Option<u64>,

    pub refetch_on_mount: bool,

    pub refetch_on_window_focus: bool,

    pub enabled: bool,
}

impl Default for QueryOptions {
    fn default() -> Self {
        Self {
            stale_time: Some(0),
            cache_time: Some(300_000),
            refetch_interval: None,
            retry: Some(3),
            retry_delay: Some(1000),
            refetch_on_mount: true,
            refetch_on_window_focus: false,
            enabled: true,
        }
    }
}

pub struct MutationCallbacks<T, V> {
    pub on_success: Option<Rc<dyn Fn(T)>>,

    pub on_error: Option<Rc<dyn Fn(String)>>,

    pub on_settled: Option<Rc<dyn Fn()>>,

    pub invalidate_keys: Vec<crate::query_key::QueryKey>,

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

use std::any::{Any, TypeId};
use std::sync::Arc;
use std::time::Instant;
use tokio::task::JoinHandle;

pub trait CacheEntry: Send + Sync {
    fn as_any(&self) -> &dyn Any;
    fn type_id(&self) -> TypeId;
    fn fetched_at(&self) -> Option<Instant>;
    fn last_accessed(&self) -> Instant;
    fn is_fetching(&self) -> bool;
    fn set_fetching(&mut self, fetching: bool);
    fn set_fetch_handle(&mut self, handle: Option<Arc<JoinHandle<()>>>);
    fn touch(&mut self);
    fn cache_time(&self) -> Option<u64>;
}

pub struct TypedCacheEntry<T: Clone + Send + Sync + 'static> {
    value: Arc<T>,
    fetched_at: Instant,
    last_accessed: Instant,
    is_fetching: bool,
    fetch_handle: Option<Arc<JoinHandle<()>>>,
    cache_time_ms: Option<u64>, // Per-entry cache time
}

impl<T: Clone + Send + Sync + 'static> TypedCacheEntry<T> {
    pub fn new(value: Arc<T>) -> Self {
        Self::with_cache_time(value, Some(300_000))
    }

    pub fn with_cache_time(value: Arc<T>, cache_time_ms: Option<u64>) -> Self {
        let now = Instant::now();
        Self {
            value,
            fetched_at: now,
            last_accessed: now,
            is_fetching: false,
            fetch_handle: None,
            cache_time_ms,
        }
    }

    pub fn placeholder(default_value: Arc<T>) -> Self {
        let now = Instant::now();
        Self {
            value: default_value,
            fetched_at: now,
            last_accessed: now,
            is_fetching: true,
            fetch_handle: None,
            cache_time_ms: Some(300_000),
        }
    }

    pub fn value(&self) -> &Arc<T> {
        &self.value
    }
}

impl<T: Clone + Send + Sync + 'static> CacheEntry for TypedCacheEntry<T> {
    fn as_any(&self) -> &dyn Any {
        &*self.value
    }

    fn type_id(&self) -> TypeId {
        TypeId::of::<T>()
    }

    fn fetched_at(&self) -> Option<Instant> {
        Some(self.fetched_at)
    }

    fn last_accessed(&self) -> Instant {
        self.last_accessed
    }

    fn is_fetching(&self) -> bool {
        self.is_fetching
    }

    fn set_fetching(&mut self, fetching: bool) {
        self.is_fetching = fetching;
    }

    fn set_fetch_handle(&mut self, handle: Option<Arc<JoinHandle<()>>>) {
        self.fetch_handle = handle;
    }

    fn touch(&mut self) {
        self.last_accessed = Instant::now();
    }

    fn cache_time(&self) -> Option<u64> {
        self.cache_time_ms
    }
}

/// Helper to retrieve typed value with runtime type checking
///
/// Returns None if the type doesn't match or if downcast fails.
pub fn get_typed_value<T: Clone + Send + Sync + 'static>(entry: &dyn CacheEntry) -> Option<Arc<T>> {
    if entry.type_id() == TypeId::of::<T>() {
        entry
            .as_any()
            .downcast_ref::<T>()
            .map(|t| Arc::new(t.clone()))
    } else {
        None
    }
}

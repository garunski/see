use crate::cache::storage::QUERY_CACHE;
use tracing::debug;

pub fn get_cache_stats() -> (usize, Vec<String>) {
    let cache = QUERY_CACHE();
    let map_ref = cache.borrow();
    let size = map_ref.len();
    let keys: Vec<String> = map_ref.keys().map(|k| k.as_str().to_string()).collect();

    debug!(
        cache_size = size,
        keys = ?keys,
        "Cache statistics retrieved"
    );

    (size, keys)
}

use tracing::{debug, info, instrument, trace};

use crate::cache::storage::QUERY_CACHE;
use crate::query_key::QueryKey;

#[instrument]
pub fn invalidate_query(key: &QueryKey) {
    info!(key = %key, "Invalidating query");
    let cache = QUERY_CACHE();
    let mut cache_map = cache.borrow_mut();
    if cache_map.remove(key).is_some() {
        debug!(key = %key, "Query removed from cache");
    } else {
        trace!(key = %key, "Query not found in cache");
    }
}

#[instrument]
pub fn invalidate_queries_by_prefix(prefix: &str) {
    info!(prefix = prefix, "Invalidating queries by prefix");
    let cache = QUERY_CACHE();
    let mut cache_map = cache.borrow_mut();
    let before_count = cache_map.len();
    cache_map.retain(|k, _| !k.as_str().starts_with(prefix));
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
    let cache = QUERY_CACHE();
    let mut cache_map = cache.borrow_mut();
    let count = cache_map.len();
    cache_map.clear();
    debug!(cleared_count = count, "All queries invalidated");
}

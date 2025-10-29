use std::collections::HashMap;
use std::time::Instant;
use tracing::{debug, info};

use crate::cache::entry::CacheEntry;
use crate::query_key::QueryKey;

/// Maximum cache size before LRU eviction kicks in
const MAX_CACHE_SIZE: usize = 1000;

/// Start the periodic cache cleanup task
///
/// NOTE: This is now a no-op. Cleanup happens lazily during cache operations
/// to avoid requiring a Dioxus runtime context in background tasks.
pub fn start_cleanup_task() {
    // No-op: cleanup now happens lazily during use_query calls
}

/// Clean up stale cache entries and enforce size limits
///
/// This should be called from within Dioxus context (e.g., during use_query)
/// Pass the cache map directly to avoid GlobalSignal access issues
pub fn cleanup_stale_entries_sync(map: &mut HashMap<QueryKey, Box<dyn CacheEntry>>) {
    let now = Instant::now();
    let mut to_remove: Vec<QueryKey> = Vec::new();

    // Find entries to remove based on their individual cache_time
    for (key, entry) in map.iter() {
        if let Some(fetched_at) = entry.fetched_at() {
            // Use per-entry cache_time (defaults to 5 minutes if not set)
            let cache_time_ms = entry.cache_time().unwrap_or(300_000);
            let cache_time = std::time::Duration::from_millis(cache_time_ms);

            if now.duration_since(fetched_at) > cache_time {
                to_remove.push(key.clone());
            }
        }
    }

    // Remove expired entries
    for key in &to_remove {
        map.remove(key);
        debug!(key = %key, "Removed expired cache entry");
    }

    // If cache is still too large, use LRU eviction
    if map.len() > MAX_CACHE_SIZE {
        evict_lru_entries(map);
    }

    if !to_remove.is_empty() || map.len() > MAX_CACHE_SIZE {
        info!(
            removed = to_remove.len(),
            current_size = map.len(),
            "Cache cleanup completed"
        );
    }
}

/// Evict least-recently-used entries until cache is under MAX_CACHE_SIZE
fn evict_lru_entries(map: &mut HashMap<QueryKey, Box<dyn CacheEntry>>) {
    // Collect all entries with their last_accessed time
    let mut entries: Vec<(QueryKey, Instant)> = map
        .iter()
        .map(|(k, v)| (k.clone(), v.last_accessed()))
        .collect();

    // Sort by last_accessed (oldest first)
    entries.sort_by_key(|(_, accessed)| *accessed);

    // Remove oldest entries until we're under the limit
    let target_size = MAX_CACHE_SIZE - (MAX_CACHE_SIZE / 10); // Remove 10% extra for buffer
    let to_remove = entries.len().saturating_sub(target_size);

    for i in 0..to_remove {
        if let Some((key, _)) = entries.get(i) {
            map.remove(key);
            debug!(key = %key, "Evicted LRU cache entry");
        }
    }

    info!(
        evicted = to_remove,
        new_size = map.len(),
        "LRU eviction completed"
    );
}

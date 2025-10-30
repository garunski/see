use std::collections::HashMap;
use std::time::Instant;
use tracing::{debug, info};

use crate::cache::entry::CacheEntry;
use crate::query_key::QueryKey;

const MAX_CACHE_SIZE: usize = 1000;

pub fn start_cleanup_task() {}

pub fn cleanup_stale_entries_sync(map: &mut HashMap<QueryKey, Box<dyn CacheEntry>>) {
    let now = Instant::now();
    let mut to_remove: Vec<QueryKey> = Vec::new();

    for (key, entry) in map.iter() {
        if let Some(fetched_at) = entry.fetched_at() {
            let cache_time_ms = entry.cache_time().unwrap_or(300_000);
            let cache_time = std::time::Duration::from_millis(cache_time_ms);

            if now.duration_since(fetched_at) > cache_time {
                to_remove.push(key.clone());
            }
        }
    }

    for key in &to_remove {
        map.remove(key);
        debug!(key = %key, "Removed expired cache entry");
    }

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

fn evict_lru_entries(map: &mut HashMap<QueryKey, Box<dyn CacheEntry>>) {
    let mut entries: Vec<(QueryKey, Instant)> = map
        .iter()
        .map(|(k, v)| (k.clone(), v.last_accessed()))
        .collect();

    entries.sort_by_key(|(_, accessed)| *accessed);

    let target_size = MAX_CACHE_SIZE - (MAX_CACHE_SIZE / 10);
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

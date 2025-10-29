use tracing::trace;

use crate::query_key::QueryKey;

/// Mark a fetch as complete (cleanup helper)
///
/// Note: Deduplication is now handled by the `is_fetching` flag in the cache entry itself.
/// This function is kept for future use if we need additional fetch tracking.
pub fn mark_fetch_complete(key: &QueryKey) {
    trace!(key = %key, "Fetch marked as complete");
    // Note: The actual is_fetching flag is cleared when cache entry is updated
    // This is just for logging/tracing purposes
}

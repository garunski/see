use tracing::trace;

use crate::query_key::QueryKey;

pub fn mark_fetch_complete(key: &QueryKey) {
    trace!(key = %key, "Fetch marked as complete");
}

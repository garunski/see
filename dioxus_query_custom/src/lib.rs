//! Custom Dioxus Query System
//!
//! A type-safe, in-memory query caching system for Dioxus with no serialization overhead.

pub mod cache;
pub mod invalidate;
pub mod mutation;
pub mod query;
pub mod query_key;
pub mod state;
pub mod utils;

// Minimal public surface
pub mod prelude {
    pub use crate::query::use_query;
    // use_queries is deprecated - do not export
    pub use crate::invalidate::{
        invalidate_all_queries, invalidate_queries_by_prefix, invalidate_query,
    };
    pub use crate::mutation::use_mutation;
    pub use crate::query_key::QueryKey;
    pub use crate::state::{MutationCallbacks, MutationState, QueryOptions, QueryState};
    pub use crate::utils::get_cache_stats;
}

use dioxus::prelude::*;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use tracing::debug;

use crate::cache::entry::CacheEntry;
use crate::query_key::QueryKey;

/// Global cache for query results (type-erased)
/// Stores the actual typed values as Arc<T> with no serialization
pub static QUERY_CACHE: GlobalSignal<Rc<RefCell<HashMap<QueryKey, Box<dyn CacheEntry>>>>> =
    Signal::global(|| {
        debug!("Initializing global query cache");
        Rc::new(RefCell::new(HashMap::new()))
    });

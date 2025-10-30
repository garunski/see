use dioxus::prelude::*;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use tracing::debug;

use crate::cache::entry::CacheEntry;
use crate::query_key::QueryKey;

pub type QueryCacheType = Rc<RefCell<HashMap<QueryKey, Box<dyn CacheEntry>>>>;

pub static QUERY_CACHE: GlobalSignal<QueryCacheType> = Signal::global(|| {
    debug!("Initializing global query cache");
    Rc::new(RefCell::new(HashMap::new()))
});

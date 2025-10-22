use crate::errors::CoreError;
use crate::persistence::store::{AuditStore, RedbStore};
use std::sync::{Arc, Mutex, Once};

static GLOBAL_STORE: Mutex<Option<Arc<dyn AuditStore + Send + Sync>>> = Mutex::new(None);
static INIT: Once = Once::new();

pub fn get_global_store() -> Result<Arc<dyn AuditStore + Send + Sync>, Box<CoreError>> {
    INIT.call_once(|| {
        let store = match RedbStore::new_default() {
            Ok(store) => Some(Arc::new(store) as Arc<dyn AuditStore + Send + Sync>),
            Err(e) => {
                eprintln!("Failed to initialize global store: {}", e);
                None
            }
        };
        if let Ok(mut global_store) = GLOBAL_STORE.lock() {
            *global_store = store;
        }
    });

    let global_store = GLOBAL_STORE.lock().map_err(|e| {
        Box::new(CoreError::MutexLock(format!(
            "Failed to lock global store: {}",
            e
        )))
    })?;
    global_store
        .as_ref()
        .ok_or_else(|| {
            Box::new(CoreError::Dataflow(
                "Global store not initialized".to_string(),
            ))
        })
        .cloned()
}

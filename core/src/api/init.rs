// Initialization functions ONLY

use crate::store_singleton;
use tracing_appender::non_blocking::WorkerGuard;

/// Type alias for tracing worker guard
pub type TracingGuard = WorkerGuard;

/// Initialize the tracing/logging system
pub fn init_tracing(log_file: Option<String>) -> Result<TracingGuard, String> {
    use tracing_subscriber::{fmt, prelude::*, EnvFilter};
    
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info"));
    
    let registry = tracing_subscriber::registry().with(filter);
    
    match log_file {
        Some(path) => {
            let file_appender = tracing_appender::rolling::daily(path, "app.log");
            let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);
            
            registry
                .with(fmt::layer().with_writer(non_blocking))
                .init();
            
            Ok(guard)
        }
        None => {
            registry
                .with(fmt::layer())
                .init();
            
            // For stdout logging, we don't need a guard
            // Return a dummy guard that does nothing
            let (_, guard) = tracing_appender::non_blocking(std::io::stdout());
            Ok(guard)
        }
    }
}

/// Initialize the global persistence store singleton
pub async fn init_global_store() -> Result<(), String> {
    store_singleton::init_global_store().await
}

/// Get reference to the global persistence store
pub fn get_global_store() -> Result<std::sync::Arc<persistence::Store>, String> {
    store_singleton::get_global_store()
}

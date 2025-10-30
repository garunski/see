use std::path::Path;
use tracing::{debug, error, info, instrument, warn};
use tracing_appender::{non_blocking, rolling};
use tracing_subscriber::{fmt, EnvFilter};

pub fn init_logging(
    log_file: Option<&Path>,
) -> Result<non_blocking::WorkerGuard, Box<dyn std::error::Error>> {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    let (writer, guard) = if let Some(log_path) = log_file {
        let file_appender = rolling::daily(log_path.parent().unwrap(), "persistence.log");
        non_blocking(file_appender)
    } else {
        non_blocking(std::io::stdout())
    };

    fmt::Subscriber::builder()
        .with_env_filter(filter)
        .with_writer(writer)
        .with_target(false)
        .with_thread_ids(true)
        .with_thread_names(true)
        .init();

    info!("Logging initialized");
    Ok(guard)
}

#[instrument(skip_all)]
pub fn log_db_operation_start(operation: &str, table: &str) {
    info!("Starting {} on {}", operation, table);
}

#[instrument(skip_all)]
pub fn log_db_operation_success(operation: &str, table: &str, duration_ms: u64) {
    info!("Completed {} on {} in {}ms", operation, table, duration_ms);
}

#[instrument(skip_all)]
pub fn log_db_operation_error(operation: &str, table: &str, error: &str) {
    error!("Failed {} on {}: {}", operation, table, error);
}

#[instrument(skip_all)]
pub fn log_serialization(operation: &str, size_bytes: usize) {
    debug!("Serialized {} ({} bytes)", operation, size_bytes);
}

#[instrument(skip_all)]
pub fn log_deserialization(operation: &str, size_bytes: usize) {
    debug!("Deserialized {} ({} bytes)", operation, size_bytes);
}

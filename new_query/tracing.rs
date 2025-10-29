use tracing::{Level, info};
use tracing_subscriber::{
    fmt,
    layer::SubscriberExt,
    util::SubscriberInitExt,
    EnvFilter,
};
use tracing_appender::{non_blocking, rolling};

/// Initialize tracing with console output and file logging
pub fn init_tracing() {
    // Create a file appender that rotates daily
    let file_appender = rolling::daily("./logs", "dioxus_query.log");
    let (non_blocking_file, _guard) = non_blocking(file_appender);

    // Console layer with colors and formatting
    let console_layer = fmt::layer()
        .with_target(true)
        .with_thread_ids(true)
        .with_line_number(true)
        .pretty();

    // File layer with JSON formatting for structured logs
    let file_layer = fmt::layer()
        .with_writer(non_blocking_file)
        .json()
        .with_current_span(true)
        .with_span_list(true);

    // Environment filter - defaults to INFO, can be overridden with RUST_LOG
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| {
            EnvFilter::new("info")
                // Enable DEBUG for query operations specifically
                .add_directive("query=debug".parse().unwrap())
        });

    // Compose and initialize subscriber
    tracing_subscriber::registry()
        .with(filter)
        .with(console_layer)
        .with(file_layer)
        .init();

    info!("Tracing initialized");
}

/// Initialize tracing with custom level
pub fn init_tracing_with_level(level: Level) {
    let file_appender = rolling::daily("./logs", "dioxus_query.log");
    let (non_blocking_file, _guard) = non_blocking(file_appender);

    let console_layer = fmt::layer()
        .with_target(true)
        .pretty();

    let file_layer = fmt::layer()
        .with_writer(non_blocking_file)
        .json();

    let filter = EnvFilter::new(level.to_string())
        .add_directive(format!("query={}", level).parse().unwrap());

    tracing_subscriber::registry()
        .with(filter)
        .with(console_layer)
        .with(file_layer)
        .init();

    info!(level = %level, "Tracing initialized with custom level");
}

/// Initialize tracing for development (verbose console, no file)
pub fn init_tracing_dev() {
    let console_layer = fmt::layer()
        .with_target(true)
        .with_thread_ids(true)
        .with_line_number(true)
        .with_file(true)
        .pretty();

    let filter = EnvFilter::new("trace")
        .add_directive("query=trace".parse().unwrap())
        .add_directive("tokio=info".parse().unwrap())
        .add_directive("hyper=info".parse().unwrap());

    tracing_subscriber::registry()
        .with(filter)
        .with(console_layer)
        .init();

    info!("Tracing initialized for development (verbose mode)");
}

/// Initialize tracing for production (structured JSON logs only)
pub fn init_tracing_prod() {
    let file_appender = rolling::daily("./logs", "prod.log");
    let (non_blocking_file, _guard) = non_blocking(file_appender);

    let file_layer = fmt::layer()
        .with_writer(non_blocking_file)
        .json()
        .with_current_span(true)
        .with_span_list(true)
        .with_thread_ids(true)
        .with_thread_names(true);

    let filter = EnvFilter::new("info")
        .add_directive("query=info".parse().unwrap())
        .add_directive("tokio=warn".parse().unwrap())
        .add_directive("hyper=warn".parse().unwrap());

    tracing_subscriber::registry()
        .with(filter)
        .with(file_layer)
        .init();

    info!("Tracing initialized for production");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tracing_init() {
        init_tracing_dev();
        tracing::info!("Test log message");
        tracing::debug!("Debug message");
        tracing::trace!("Trace message");
    }
}
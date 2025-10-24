use crate::errors::CoreError;
use std::path::PathBuf;

pub struct TracingGuard {
    _guard: tracing_appender::non_blocking::WorkerGuard,
}

pub fn init_tracing(log_dir: Option<PathBuf>) -> Result<TracingGuard, Box<CoreError>> {
    use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

    let log_dir = match log_dir {
        Some(dir) => dir,
        None => {
            let home = dirs::home_dir().ok_or_else(|| {
                Box::new(CoreError::Dataflow(
                    "Could not find home directory".to_string(),
                ))
            })?;
            home.join(".s_e_e").join("logs")
        }
    };

    std::fs::create_dir_all(&log_dir).map_err(|e| Box::new(CoreError::from(e)))?;

    let file_appender = tracing_appender::rolling::daily(log_dir, "s_e_e.log");
    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    tracing_subscriber::registry()
        .with(env_filter)
        .with(
            tracing_subscriber::fmt::layer()
                .with_target(true)
                .with_thread_ids(false)
                .with_line_number(true),
        )
        .with(
            tracing_subscriber::fmt::layer()
                .json()
                .with_writer(non_blocking),
        )
        .init();

    Ok(TracingGuard { _guard: guard })
}

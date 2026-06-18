use anyhow::Result;
use tracing_subscriber::{EnvFilter, Layer, layer::SubscriberExt, util::SubscriberInitExt};

use crate::paths::RelayPaths;

pub struct LoggingGuard {
    _file_guard: tracing_appender::non_blocking::WorkerGuard,
}

pub fn init(paths: &RelayPaths) -> Result<LoggingGuard> {
    paths.ensure()?;

    let file_appender = tracing_appender::rolling::daily(&paths.log_dir, "relay.log");
    let (file_writer, file_guard) = tracing_appender::non_blocking(file_appender);
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    let file_layer = tracing_subscriber::fmt::layer()
        .with_writer(file_writer)
        .with_ansi(false)
        .with_filter(filter.clone());

    let stderr_layer = tracing_subscriber::fmt::layer()
        .with_writer(std::io::stderr)
        .with_filter(filter);

    tracing_subscriber::registry()
        .with(file_layer)
        .with(stderr_layer)
        .init();

    tracing::info!(
        data_dir = %paths.data_dir.display(),
        config_dir = %paths.config_dir.display(),
        log_dir = %paths.log_dir.display(),
        "Relay logging initialized"
    );

    Ok(LoggingGuard {
        _file_guard: file_guard,
    })
}

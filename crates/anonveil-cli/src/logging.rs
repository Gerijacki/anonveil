//! `tracing` setup: a rotating file log at `/var/log/anonveil/`, plus
//! terse stderr output for warnings/errors only (the CLI's own `println!`
//! output is the primary UX — logs are for debugging/audit, not chat).

use tracing_subscriber::{fmt, prelude::*, EnvFilter};

const LOG_DIR: &str = "/var/log/anonveil";
const LOG_FILE_PREFIX: &str = "anonveil.log";

/// Initialize logging. Falls back to stderr-only if `/var/log/anonveil`
/// can't be created (e.g. not running as root yet, as with `--help`).
pub fn init(level: &str) -> Option<tracing_appender::non_blocking::WorkerGuard> {
    let env_filter = EnvFilter::try_new(level).unwrap_or_else(|_| EnvFilter::new("info"));

    let file_layer_and_guard = std::fs::create_dir_all(LOG_DIR).ok().map(|_| {
        let file_appender = tracing_appender::rolling::daily(LOG_DIR, LOG_FILE_PREFIX);
        tracing_appender::non_blocking(file_appender)
    });

    let stderr_layer = fmt::layer()
        .with_writer(std::io::stderr)
        .with_target(false)
        .with_level(true)
        .compact();

    match file_layer_and_guard {
        Some((non_blocking, guard)) => {
            let file_layer = fmt::layer()
                .with_writer(non_blocking)
                .with_ansi(false)
                .with_target(true);
            tracing_subscriber::registry()
                .with(env_filter)
                .with(stderr_layer)
                .with(file_layer)
                .init();
            Some(guard)
        }
        None => {
            tracing_subscriber::registry()
                .with(env_filter)
                .with(stderr_layer)
                .init();
            None
        }
    }
}

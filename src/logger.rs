use crate::prelude::*;

const LOG_DIR: &str = "logs";
const MAX_LOG_AGE: Duration = Duration::from_secs(60 * 24 * 60 * 60); // ~2 months

/// Removes log files older than [`MAX_LOG_AGE`].
fn cleanup_old_logs() {
    tracing::info!("Cleaning up old logs...");

    let cutoff = SystemTime::now() - MAX_LOG_AGE;
    let Ok(entries) = std::fs::read_dir(LOG_DIR) else {
        return;
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let Ok(meta) = entry.metadata() else { continue };
        let Ok(modified) = meta.modified() else {
            continue;
        };
        if modified < cutoff {
            let _ = std::fs::remove_file(&path);
        }
    }
}

/// Initializes the tracing subscriber and starts the log cleanup task.
pub fn setup() -> (WorkerGuard, JoinHandle<()>) {
    let module = env!("CARGO_PKG_NAME").replace('-', "_");

    let file_appender = tracing_appender::rolling::daily(LOG_DIR, format!("{module}.log"));
    let (non_blocking_file, guard) = tracing_appender::non_blocking(file_appender);

    let registry = tracing_subscriber::registry()
        .with(EnvFilter::new(format!(
            "{module}=debug,songbird=debug,symphonia=debug"
        )))
        .with(
            tracing_subscriber::fmt::layer()
                .with_ansi(false)
                .with_writer(non_blocking_file), // daily rotating file
        );

    #[cfg(debug_assertions)]
    let registry = registry.with(tracing_subscriber::fmt::layer()); // stdout (debug only)

    registry.init();

    // clean up once immediately, then every 24h
    let cleaner = spawn(async {
        loop {
            cleanup_old_logs();
            sleep(Duration::from_secs(24 * 60 * 60)).await;
        }
    });

    (guard, cleaner)
}

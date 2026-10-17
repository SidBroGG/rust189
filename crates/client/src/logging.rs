use std::fs::File;
use std::path::Path;
use std::sync::Mutex;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{fmt, EnvFilter};

/// Initializes structured logging for both terminal console and `logs/latest.log` file.
///
/// Log format closely mirrors the clean Minecraft server/client logging format:
/// `[21:55:00 INFO] [client]: Message`
pub fn init_logging() -> anyhow::Result<()> {
    // Ensure logs directory exists
    let logs_dir = Path::new("logs");
    if !logs_dir.exists() {
        std::fs::create_dir_all(logs_dir)?;
    }

    let log_file = File::create(logs_dir.join("latest.log"))?;

    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| {
        EnvFilter::new("info")
    });

    let console_layer = fmt::layer()
        .with_target(true)
        .with_thread_ids(false)
        .with_thread_names(false)
        .with_ansi(true);

    let file_layer = fmt::layer()
        .with_target(true)
        .with_ansi(false)
        .with_writer(Mutex::new(log_file));

    tracing_subscriber::registry()
        .with(env_filter)
        .with(console_layer)
        .with(file_layer)
        .try_init()
        .map_err(|e| anyhow::anyhow!("Failed to initialize tracing subscriber: {}", e))?;

    tracing::info!("Logger initialized. Logging to terminal and logs/latest.log");
    Ok(())
}

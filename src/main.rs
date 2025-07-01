//! This is a crate documentation comment.
//! It provides documentation for the entire crate.

use clap::Parser;
use std::path::PathBuf;
use tracing::info;
use zephyrite::{Config, Server, StorageConfig};

#[derive(Parser, Debug)]
#[command(name = "zephyrite")]
#[command(about = "A high-performance key-value store")]
#[command(version = zephyrite::VERSION)]
struct Cli {
    /// Port to run the server on
    #[arg(short, long, default_value = "8080")]
    port: u16,

    /// Log level for the server
    #[arg(short, long, default_value = "info")]
    log_level: Option<String>,

    /// Enable persistent storage with Write-Ahead Log
    #[arg(long)]
    persistent: bool,

    /// Path to the WAL file (implies --persistent)
    #[arg(long, value_name = "PATH")]
    wal_file: Option<PathBuf>,

    /// Initial memory capacity for storage
    #[arg(long, value_name = "SIZE")]
    memory_capacity: Option<usize>,

    /// Disable checksums in WAL entries (only for persistent storage)
    #[arg(long)]
    no_checksums: bool,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    tracing_subscriber::fmt()
        .with_max_level(match cli.log_level.as_deref() {
            Some("trace") => tracing::Level::TRACE,
            Some("debug") => tracing::Level::DEBUG,
            Some("warn") => tracing::Level::WARN,
            Some("error") => tracing::Level::ERROR,
            _ => tracing::Level::INFO,
        })
        .init();

    info!("🚀 Starting Zephyrite v{}", zephyrite::VERSION);
    info!(
        "🔧 Log level: {}",
        cli.log_level.as_deref().unwrap_or("info")
    );

    // Determine storage configuration
    let storage_config = if cli.persistent || cli.wal_file.is_some() {
        let wal_path = cli
            .wal_file
            .unwrap_or_else(|| PathBuf::from("zephyrite.wal"));
        info!("💾 Using persistent storage with WAL file: {:?}", wal_path);

        let mut config = StorageConfig::persistent(wal_path).with_checksums(!cli.no_checksums);

        if let Some(capacity) = cli.memory_capacity {
            config = config.with_memory_capacity(capacity);
            info!("🧠 Memory capacity set to: {}", capacity);
        }

        if cli.no_checksums {
            info!("⚠️  WAL checksums disabled");
        }

        config
    } else {
        info!("⚡ Using in-memory storage (no persistence)");
        let mut config = StorageConfig::memory();

        if let Some(capacity) = cli.memory_capacity {
            config = config.with_memory_capacity(capacity);
            info!("🧠 Memory capacity set to: {}", capacity);
        }

        config
    };

    let config = Config::new_with_storage(cli.port, storage_config);
    let server = Server::new(config)?;

    server.start().await?;

    Ok(())
}

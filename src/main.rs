//! This is a crate documentation comment.
//! It provides documentation for the entire crate.

use clap::Parser;
use tracing::info;
use zephyrite::{Config, Server};

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

    let config = Config::new(cli.port);
    let server = Server::new(config);

    server.start().await?;

    Ok(())
}

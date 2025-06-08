//! HTTP server module for Zephyrite
use crate::Config;
use axum::{Router, response::Json, routing::get};
use serde::{Deserialize, Serialize};
use std::io;
use thiserror::Error;
use tracing::info;

/// Errors that can occur in the server
#[derive(Debug, Error)]
pub enum ServerError {
    /// I/O error when binding to address
    #[error("Failed to bind to address {0}")]
    AddressBindError(#[from] io::Error),

    /// Server startup error
    #[error("Server failed to start: {0}")]
    StartupError(String),
}

/// Result type for server operations
pub type Result<T> = std::result::Result<T, ServerError>;

/// HTTP Server
pub struct Server {
    config: Config,
}

impl Server {
    /// Creates a new server instance with the given configuration.
    #[must_use]
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    /// Start the server and listen for incoming requests.
    ///
    /// # Errors
    ///
    /// Returns `ServerError::AddressBindError` if the server fails to bind to the configured address
    /// or encounters an I/O error during operation.
    pub async fn start(&self) -> Result<()> {
        let app = Router::new()
            .route("/", get(health_check))
            .route("/health", get(health_check));

        info!("🌟 Starting Zephyrite server on {}", self.config.address);

        let listener = tokio::net::TcpListener::bind(&self.config.address)
            .await
            .map_err(ServerError::AddressBindError)?;

        axum::serve(listener, app)
            .await
            .map_err(ServerError::AddressBindError)?;

        Ok(())
    }
}

/// Health check res
#[derive(Serialize, Deserialize)]
pub struct HealthResponse {
    status: String,
    version: String,
    service: String,
}

async fn health_check() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".to_string(),
        version: crate::VERSION.to_string(),
        service: "Zephyrite".to_string(),
    })
}

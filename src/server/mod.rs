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
    /// # Arguments
    ///
    /// * `shutdown_signal` - Optional future that resolves when the server should shut down.
    /// * `bound_addr_tx` - Optional channel sender to communicate the actual bound address back to the test.
    ///
    /// # Errors
    ///
    /// Returns `ServerError::AddressBindError` if the server fails to bind to the configured address
    /// or encounters an I/O error during operation.
    ///
    /// # Panics
    ///
    /// Panics if retrieving the local address from the listener fails.
    pub async fn start_with_shutdown<F>(
        &self,
        shutdown_signal: Option<F>,
        bound_addr_tx: Option<tokio::sync::oneshot::Sender<std::net::SocketAddr>>,
    ) -> Result<()>
    where
        F: std::future::Future<Output = ()> + Send + 'static,
    {
        let app = Router::new()
            .route("/", get(health_check))
            .route("/health", get(health_check));

        info!("🌟 Starting Zephyrite server on {}", self.config.address);

        let listener = tokio::net::TcpListener::bind(&self.config.address)
            .await
            .map_err(ServerError::AddressBindError)?;

        // Communicate the actual bound address if a channel is provided
        if let Some(tx) = bound_addr_tx {
            let _ = tx.send(listener.local_addr().unwrap());
        }

        match shutdown_signal {
            Some(sig) => {
                axum::serve(listener, app)
                    .with_graceful_shutdown(sig)
                    .await
                    .map_err(ServerError::AddressBindError)?;
            }
            None => {
                axum::serve(listener, app)
                    .await
                    .map_err(ServerError::AddressBindError)?;
            }
        }
        Ok(())
    }

    /// Start the server and listen for incoming requests (no shutdown signal).
    ///
    /// # Errors
    ///
    /// Returns `ServerError::AddressBindError` if the server fails to bind to the configured address
    /// or encounters an I/O error during operation.
    pub async fn start(&self) -> Result<()> {
        self.start_with_shutdown::<std::future::Ready<()>>(None, None)
            .await
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

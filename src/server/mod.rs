//! HTTP server module for Zephyrite

mod handlers;
mod types;

// Re-export types for public API
pub use types::*;

use crate::{
    Config,
    storage::{MemoryStorage, StorageEngine},
};
use axum::{
    Router,
    routing::{delete, get, put},
};
use std::sync::Arc;
use tracing::info;

use handlers::{delete_key, get_key, health_check, list_keys, put_key};

/// HTTP Server with integrated storage
pub struct Server {
    config: Config,
    storage: Arc<dyn StorageEngine>,
}

impl Server {
    /// Creates a new server instance with the given configuration and default storage.
    #[must_use]
    pub fn new(config: Config) -> Self {
        Self::with_storage(config, Arc::new(MemoryStorage::new()))
    }

    /// Creates a new server instance with the given configuration and custom storage.
    #[must_use]
    pub fn with_storage(config: Config, storage: Arc<dyn StorageEngine>) -> Self {
        Self { config, storage }
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
        let app = self.create_router();

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

    /// Create the axum router with all endpoints
    fn create_router(&self) -> Router {
        Router::new()
            .route("/", get(health_check))
            .route("/health", get(health_check))
            .route("/keys", get(list_keys))
            .route("/keys/{key}", get(get_key))
            .route("/keys/{key}", put(put_key))
            .route("/keys/{key}", delete(delete_key))
            .with_state(Arc::clone(&self.storage))
    }
}

//! HTTP server module for Zephyrite
use crate::{
    Config,
    storage::{MemoryStorage, StorageEngine, StorageError},
};
use axum::{
    Router,
    extract::{Path, State},
    http::StatusCode,
    response::Json,
    routing::{delete, get, put},
};
use serde::{Deserialize, Serialize};
use std::{io, sync::Arc};
use thiserror::Error;
use tracing::{error, info};

/// Errors that can occur in the server
#[derive(Debug, Error)]
pub enum ServerError {
    /// I/O error when binding to address
    #[error("Failed to bind to address {0}")]
    AddressBindError(#[from] io::Error),

    /// Server startup error
    #[error("Server failed to start: {0}")]
    StartupError(String),

    /// Storage error
    #[error("Storage error: {0}")]
    StorageError(#[from] StorageError),
}

/// Result type for server operations
pub type Result<T> = std::result::Result<T, ServerError>;

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

/// Request body for storing a value
#[derive(Deserialize)]
pub struct PutKeyRequest {
    /// The value to store
    pub value: String,
}

/// Response for health check endpoint
#[derive(Serialize)]
pub struct HealthResponse {
    /// Status of the service
    pub status: String,
    /// Version of the service
    pub version: String,
    /// Name of the service
    pub service: String,
}

/// Response for getting a key
#[derive(Serialize)]
pub struct GetKeyResponse {
    /// The key that was requested
    pub key: String,
    /// The value associated with the key
    pub value: String,
    /// Whether the key was found
    pub found: bool,
    /// Size of the value in bytes
    pub size: usize,
    /// Creation timestamp of the key
    pub created_at: String,
    /// Last updated timestamp of the key
    pub updated_at: String,
}

/// Response for listing keys
#[derive(Serialize)]
pub struct ListKeysResponse {
    /// List of keys stored in the system
    pub keys: Vec<String>,
    /// Count of keys stored
    pub count: usize,
}

/// Error response
#[derive(Serialize)]
pub struct ErrorResponse {
    /// Error code indicating the type of error
    pub error: String,
    /// Human-readable error message
    /// This message provides more context about the error
    /// and can be used for debugging or user feedback.
    pub message: String,
}

/// Health check endpoint
async fn health_check() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".to_string(),
        version: crate::VERSION.to_string(),
        service: "Zephyrite".to_string(),
    })
}

/// GET /keys/:key - Retrieve a value by key
async fn get_key(
    Path(key): Path<String>,
    State(storage): State<Arc<dyn StorageEngine>>,
) -> std::result::Result<Json<GetKeyResponse>, (StatusCode, Json<ErrorResponse>)> {
    match storage.get(&key) {
        Ok(stored_value) => Ok(Json(GetKeyResponse {
            key: key.clone(),
            value: stored_value.value,
            found: true,
            size: stored_value.metadata.size,
            created_at: stored_value.metadata.created_at,
            updated_at: stored_value.metadata.updated_at,
        })),
        Err(StorageError::KeyNotFound(_)) => Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: "key_not_found".to_string(),
                message: format!("Key '{key}' not found"),
            }),
        )),
        Err(StorageError::InvalidKey(msg)) => Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "invalid_key".to_string(),
                message: msg,
            }),
        )),
        Err(e) => {
            error!("Storage error in get_key: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "internal_error".to_string(),
                    message: "Internal server error".to_string(),
                }),
            ))
        }
    }
}

/// PUT /keys/:key - Store a key-value pair
async fn put_key(
    Path(key): Path<String>,
    State(storage): State<Arc<dyn StorageEngine>>,
    Json(request): Json<PutKeyRequest>,
) -> std::result::Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    match storage.put(&key, &request.value) {
        Ok(was_new) => {
            if was_new {
                Ok(StatusCode::CREATED)
            } else {
                Ok(StatusCode::OK)
            }
        }
        Err(StorageError::InvalidKey(msg)) => Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "invalid_key".to_string(),
                message: msg,
            }),
        )),
        Err(StorageError::InvalidValue(msg)) => Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "invalid_value".to_string(),
                message: msg,
            }),
        )),
        Err(e) => {
            error!("Storage error in put_key: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "internal_error".to_string(),
                    message: "Internal server error".to_string(),
                }),
            ))
        }
    }
}

/// DELETE /keys/:key - Delete a key
async fn delete_key(
    Path(key): Path<String>,
    State(storage): State<Arc<dyn StorageEngine>>,
) -> std::result::Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    match storage.delete(&key) {
        Ok(existed) => {
            if existed {
                Ok(StatusCode::NO_CONTENT)
            } else {
                Ok(StatusCode::NOT_FOUND)
            }
        }
        Err(StorageError::InvalidKey(msg)) => Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "invalid_key".to_string(),
                message: msg,
            }),
        )),
        Err(e) => {
            error!("Storage error in delete_key: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "internal_error".to_string(),
                    message: "Internal server error".to_string(),
                }),
            ))
        }
    }
}

/// GET /keys - List all keys
async fn list_keys(
    State(storage): State<Arc<dyn StorageEngine>>,
) -> std::result::Result<Json<ListKeysResponse>, (StatusCode, Json<ErrorResponse>)> {
    match storage.keys() {
        Ok(keys) => Ok(Json(ListKeysResponse {
            count: keys.len(),
            keys,
        })),
        Err(e) => {
            error!("Storage error in list_keys: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "internal_error".to_string(),
                    message: "Internal server error".to_string(),
                }),
            ))
        }
    }
}

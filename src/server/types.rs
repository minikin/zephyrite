use crate::storage::StorageError;
use serde::{Deserialize, Serialize};
use std::io;
use thiserror::Error;

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

use thiserror::Error;

/// Errors that can occur during storage operation
#[derive(Debug, Clone, PartialEq, Error)]
pub enum StorageError {
    /// The requested key was not found
    #[error("Key not found: {0}")]
    KeyNotFound(String),

    /// The requested value was not found
    #[error("Key already exists: {0}")]
    KeyAlreadyExists(String),

    /// Invalid key format or content
    #[error("Invalid key: {0}")]
    InvalidKey(String),

    /// Invalid value format or content
    #[error("Invalid value: {0}")]
    InvalidValue(String),

    /// Internal storage error
    #[error("Internal storage error: {0}")]
    Internal(String),

    /// Unsupported operation or feature
    #[error("Unsupported operation: {0}")]
    UnsupportedOperation(String),
}

/// Result type for storage operations
pub type StorageResult<T> = Result<T, StorageError>;

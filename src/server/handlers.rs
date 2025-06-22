use crate::storage::utils::{validate_key, validate_value};
use crate::storage::{StorageEngine, StorageError};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};
use std::sync::Arc;
use tracing::{error, info, instrument, warn};

use super::types::{
    ErrorResponse, GetKeyResponse, HealthResponse, ListKeysResponse, PutKeyRequest,
};

type HandlerResult<T> = std::result::Result<T, (StatusCode, Json<ErrorResponse>)>;

/// Represents the different storage operations that can fail
#[derive(Debug, Clone, Copy)]
enum Operation {
    GetKey,
    PutKey,
    DeleteKey,
    ListKeys,
}

impl std::fmt::Display for Operation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Operation::GetKey => write!(f, "get_key"),
            Operation::PutKey => write!(f, "put_key"),
            Operation::DeleteKey => write!(f, "delete_key"),
            Operation::ListKeys => write!(f, "list_keys"),
        }
    }
}

/// Convert storage errors to HTTP responses
fn handle_storage_error(
    error: StorageError,
    operation: Operation,
) -> (StatusCode, Json<ErrorResponse>) {
    match error {
        StorageError::KeyNotFound(key) => (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: "key_not_found".to_string(),
                message: format!("Key '{key}' not found"),
            }),
        ),
        StorageError::InvalidKey(msg) => (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "invalid_key".to_string(),
                message: msg,
            }),
        ),
        StorageError::InvalidValue(msg) => (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "invalid_value".to_string(),
                message: msg,
            }),
        ),
        e => {
            error!("Storage error in {}: {}", operation, e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "internal_error".to_string(),
                    message: "Internal server error".to_string(),
                }),
            )
        }
    }
}

/// Health check endpoint
#[instrument]
pub async fn health_check() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".to_string(),
        version: crate::VERSION.to_string(),
        service: "Zephyrite".to_string(),
    })
}

/// GET /keys/:key - Retrieve a value by key
#[instrument(skip(storage))]
pub async fn get_key(
    Path(key): Path<String>,
    State(storage): State<Arc<dyn StorageEngine>>,
) -> HandlerResult<Json<GetKeyResponse>> {
    if let Err(e) = validate_key(&key) {
        return Err(handle_storage_error(e, Operation::GetKey));
    }

    info!("Retrieving key: {}", key);

    match storage.get(&key) {
        Ok(stored_value) => {
            info!(
                "Successfully retrieved key: {}, size: {} bytes",
                key, stored_value.metadata.size
            );
            Ok(Json(GetKeyResponse {
                key: key.clone(),
                value: stored_value.value,
                found: true,
                size: stored_value.metadata.size,
                created_at: stored_value.metadata.created_at,
                updated_at: stored_value.metadata.updated_at,
            }))
        }
        Err(StorageError::KeyNotFound(_)) => {
            warn!("Key not found: {}", key);
            Err(handle_storage_error(
                StorageError::KeyNotFound(key.clone()),
                Operation::GetKey,
            ))
        }
        Err(e) => Err(handle_storage_error(e, Operation::GetKey)),
    }
}

/// PUT /keys/:key - Store a key-value pair
#[instrument(skip(storage, request))]
pub async fn put_key(
    Path(key): Path<String>,
    State(storage): State<Arc<dyn StorageEngine>>,
    Json(request): Json<PutKeyRequest>,
) -> HandlerResult<StatusCode> {
    if let Err(e) = validate_key(&key) {
        return Err(handle_storage_error(e, Operation::PutKey));
    }

    if let Err(e) = validate_value(&request.value) {
        return Err(handle_storage_error(e, Operation::PutKey));
    }

    let value_size = request.value.len();
    info!("Storing key: {}, value size: {} bytes", key, value_size);

    match storage.put(&key, &request.value) {
        Ok(was_new) => {
            if was_new {
                info!("Successfully created new key: {}", key);
                Ok(StatusCode::CREATED)
            } else {
                info!("Successfully updated existing key: {}", key);
                Ok(StatusCode::OK)
            }
        }
        Err(e) => Err(handle_storage_error(e, Operation::PutKey)),
    }
}

/// DELETE /keys/:key - Delete a key
#[instrument(skip(storage))]
pub async fn delete_key(
    Path(key): Path<String>,
    State(storage): State<Arc<dyn StorageEngine>>,
) -> HandlerResult<StatusCode> {
    if let Err(e) = validate_key(&key) {
        return Err(handle_storage_error(e, Operation::DeleteKey));
    }

    info!("Deleting key: {}", key);

    match storage.delete(&key) {
        Ok(existed) => {
            if existed {
                info!("Successfully deleted key: {}", key);
                Ok(StatusCode::NO_CONTENT)
            } else {
                warn!("Attempted to delete non-existent key: {}", key);
                Ok(StatusCode::NOT_FOUND)
            }
        }
        Err(e) => Err(handle_storage_error(e, Operation::DeleteKey)),
    }
}

/// GET /keys - List all keys
#[instrument(skip(storage))]
pub async fn list_keys(
    State(storage): State<Arc<dyn StorageEngine>>,
) -> HandlerResult<Json<ListKeysResponse>> {
    info!("Listing all keys");

    match storage.keys() {
        Ok(keys) => {
            info!("Successfully retrieved {} keys", keys.len());
            Ok(Json(ListKeysResponse {
                count: keys.len(),
                keys,
            }))
        }
        Err(e) => Err(handle_storage_error(e, Operation::ListKeys)),
    }
}

use crate::storage::{StorageEngine, StorageError};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};
use std::sync::Arc;
use tracing::error;

use super::types::{
    ErrorResponse, GetKeyResponse, HealthResponse, ListKeysResponse, PutKeyRequest,
};

/// Health check endpoint
pub async fn health_check() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".to_string(),
        version: crate::VERSION.to_string(),
        service: "Zephyrite".to_string(),
    })
}

/// GET /keys/:key - Retrieve a value by key
pub async fn get_key(
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
pub async fn put_key(
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
pub async fn delete_key(
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
pub async fn list_keys(
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

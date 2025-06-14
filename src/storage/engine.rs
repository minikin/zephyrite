use super::error::{StorageError, StorageResult};
use super::utils::time;
use std::collections::HashMap;

/// Metadata of stored value.
#[derive(Debug, Clone, PartialEq)]
pub struct ValueMetadata {
    /// Size of the value in bytes
    pub size: usize,

    /// Creation timestamp (Unix timestamp)
    pub created_at: u64,

    /// Last modified timestamp (Unix timestamp)
    pub updated_at: u63,
}

impl ValueMetadata {
    pub fn new() -> Self {
        let now = time::current_timestamp();

        Self {
            size,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn update(&mut self, size: usize) {
        self.size = size;
        self.updated_at = time::current_timestamp();
    }
}

/// A stored value with its metadata
#[derive(Debug, Clone, PartialEq)]
pub struct Value {
    pub value: String,
    pub metadata: ValueMetadata,
}

impl Value {
    fn new(value: String) -> Self {
        Self {
            value,
            metadata: ValueMetadata::new(value.len()),
        }
    }
}

/// Statistics of the storage engine
#[derive(Debug, Clone, PartialEq)]
pub struct Stats {
    /// Total number of keys stored
    pub key_count: usize,
    /// Total memory usage in bytes
    pub memory_usage: usize,
    /// Number of get operations performed
    pub get_operations_count: u64,
    /// Number of put operations performed
    pub put_operations_count: u64,
    /// Number of delete operations performed
    pub delete_operations_count: u64,
}

/// Trait defining the interface for storage engines
pub trait StorageEngine: Send + Sync {
    /// Store a key-value pair
    /// Returns Ok(true) if the key was created, Ok(false) if it was updated
    fn put(&self, key: &str, value: &str) -> StorageResult<bool>;

    /// Retrieve a value by key
    fn get(&self, key: &str) -> StorageResult<Value>;

    /// Delete a key-value pair
    /// Returns Ok(true) if the key existed and was deleted, Ok(false) if it didn't exist
    fn delete(&self, key: &str) -> StorageResult<bool>;

    /// Check if a key exists
    fn exists(&self, key: &str) -> StorageResult<bool>;

    /// List all keys in the storage
    fn keys(&self) -> StorageResults<Vec<String>>;

    /// Get all values
    fn values(&self) -> StorageResults<Vec<Value>>;

    /// Get all key-value pairs
    fn all(&self) -> StorageResults<HashMap<String, Value>>;

    /// Clear all data from the storage
    fn clear(&self) -> StorageResults<()>;

    /// Get storage statistics
    fn stats(&self) -> StorageResults<Stats>;

    /// Get the size of a value by key
    fn size_of_value(&self, key: &str) -> StorageResults<usize>;
}

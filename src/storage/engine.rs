use super::error::StorageResult;
use crate::utils::time;
use std::collections::HashMap;

/// Metadata of stored value.
#[derive(Debug, Clone, PartialEq)]
pub struct ValueMetadata {
    /// Size of the value in bytes
    pub size: usize,

    /// Creation timestamp (Unix timestamp)
    pub created_at: String,

    /// Last modified timestamp (Unix timestamp)
    pub updated_at: String,
}

impl ValueMetadata {
    /// Creates new metadata with the given size and current timestamp
    #[must_use]
    pub fn new(size: usize) -> Self {
        let timestamp = time::current_timestamp();

        Self {
            size,
            created_at: timestamp.clone(),
            updated_at: timestamp,
        }
    }
    /// Updates the metadata with a new size and updates the timestamp
    pub fn update(&mut self, size: usize) {
        self.size = size;
        self.updated_at = time::current_timestamp();
    }
}

/// A stored value with its metadata
#[derive(Debug, Clone, PartialEq)]
pub struct Value {
    /// The actual string value being stored
    pub value: String,
    /// Metadata associated with the value (size, timestamps, etc.)
    pub metadata: ValueMetadata,
}

impl Value {
    /// Creates a new Value with the given string and metadata
    #[must_use]
    pub fn new(value: String) -> Self {
        let size = value.len();
        Self {
            value,
            metadata: ValueMetadata::new(size),
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
    ///
    /// # Errors
    /// Returns an error if the storage operation fails
    fn put(&self, key: &str, value: &str) -> StorageResult<bool>;

    /// Retrieve a value by key
    ///
    /// # Errors
    /// Returns an error if the key is not found or the storage operation fails
    fn get(&self, key: &str) -> StorageResult<Value>;

    /// Delete a key-value pair
    /// Returns Ok(true) if the key existed and was deleted, Ok(false) if it didn't exist
    ///
    /// # Errors
    /// Returns an error if the storage operation fails
    fn delete(&self, key: &str) -> StorageResult<bool>;

    /// Check if a key exists
    ///
    /// # Errors
    /// Returns an error if the storage operation fails
    fn exists(&self, key: &str) -> StorageResult<bool>;

    /// List all keys in the storage
    ///
    /// # Errors
    /// Returns an error if the storage operation fails
    fn keys(&self) -> StorageResult<Vec<String>>;

    /// Get all values
    ///
    /// # Errors
    /// Returns an error if the storage operation fails
    fn values(&self) -> StorageResult<Vec<Value>>;

    /// Get all key-value pairs
    ///
    /// # Errors
    /// Returns an error if the storage operation fails
    fn all(&self) -> StorageResult<HashMap<String, Value>>;

    /// Clear all data from the storage
    ///
    /// # Errors
    /// Returns an error if the storage operation fails
    fn clear(&self) -> StorageResult<()>;

    /// Get storage statistics
    ///
    /// # Errors
    /// Returns an error if the storage operation fails
    fn stats(&self) -> StorageResult<Stats>;

    /// Get the size of a value by key
    ///
    /// # Errors
    /// Returns an error if the key is not found or the storage operation fails
    fn size_of_value(&self, key: &str) -> StorageResult<usize>;
}

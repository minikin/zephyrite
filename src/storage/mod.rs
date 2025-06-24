//! Storage module for Zephyrite key-value database
//!
//! This module provides the core storage functionality for Zephyrite, including:
//! - Storage engine trait definitions
//! - In-memory storage implementation
//! - Error handling for storage operations
//!
//! # Example Usage
//!
//! ```rust
//! use zephyrite::storage::{MemoryStorage, StorageEngine};
//!
//! let storage = MemoryStorage::new();
//!
//! // Store a value
//! storage.put("hello", "world").unwrap();
//!
//! // Retrieve a value
//! let stored_value = storage.get("hello").unwrap();
//! assert_eq!(stored_value.value, "world");
//!
//! // List all keys
//! let keys = storage.keys().unwrap();
//! assert_eq!(keys.len(), 1);
//! ```

/// Storage engine trait and core types
pub mod engine;
/// Error types for storage operations
pub mod error;
/// In-memory storage implementation
pub mod memory;
/// Utility functions for storage operations
pub mod utils;
/// Write-ahead log (WAL) implementation
pub mod wal;

pub use engine::{Stats, StorageEngine, Value, ValueMetadata};
pub use error::{StorageError, StorageResult};
pub use memory::MemoryStorage;

/// Create a new default storage engine
///
/// Currently returns a `MemoryStorage` instance, but this can be extended
/// to support configuration-based storage engine selection in the future.
#[must_use]
pub fn storage() -> Box<dyn StorageEngine> {
    Box::new(MemoryStorage::new())
}

/// Create a storage engine with the specified capacity
///
/// For memory storage, this pre-allocates the internal `HashMap` capacity.
/// For future disk-based storage, this might set cache sizes or other parameters.
#[must_use]
pub fn storage_with_capacity(capacity: usize) -> Box<dyn StorageEngine> {
    Box::new(MemoryStorage::with_capacity(capacity))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_default_storage() {
        let storage = storage();
        let stats = storage.stats().unwrap();
        assert_eq!(stats.key_count, 0);
    }

    #[test]
    fn test_create_storage_with_capacity() {
        let storage = storage_with_capacity(100);
        let stats = storage.stats().unwrap();
        assert_eq!(stats.key_count, 0);
    }

    #[test]
    fn test_basic_operations_through_trait() {
        let storage = storage();

        storage.put("test", "value").unwrap();
        assert!(storage.exists("test").unwrap());

        let retrieved = storage.get("test").unwrap();
        assert_eq!(retrieved.value, "value");

        let deleted = storage.delete("test").unwrap();
        assert!(deleted);
        assert!(!storage.exists("test").unwrap());
    }
}

use crate::storage::Stats;
use crate::storage::utils::validate_value;

use super::engine::{StorageEngine, Value};
use super::error::{StorageError, StorageResult};
use super::utils::validate_key;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, RwLock};

/// In-memory storage engine implementation
#[derive(Debug, Default)]
pub struct MemoryStorage {
    data: Arc<RwLock<HashMap<String, Value>>>,
    get_ops: AtomicU64,
    put_ops: AtomicU64,
    delete_ops: AtomicU64,
}

impl MemoryStorage {
    /// Create a new in-memory storage
    #[must_use]
    pub fn new() -> Self {
        Self {
            data: Arc::new(RwLock::new(HashMap::new())),
            get_ops: AtomicU64::new(0),
            put_ops: AtomicU64::new(0),
            delete_ops: AtomicU64::new(0),
        }
    }

    /// Create a new in-memory storage with initial capacity
    #[must_use]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            data: Arc::new(RwLock::new(HashMap::with_capacity(capacity))),
            get_ops: AtomicU64::new(0),
            put_ops: AtomicU64::new(0),
            delete_ops: AtomicU64::new(0),
        }
    }

    /// Calculate memory usage of the current data
    #[must_use]
    pub fn calculate_memory_usage(data: &HashMap<String, Value>) -> usize {
        data.iter()
            .map(|(key, value)| key.len() + value.value.len() + std::mem::size_of::<Value>())
            .sum()
    }
}

impl StorageEngine for MemoryStorage {
    fn put(&self, key: &str, value: &str) -> StorageResult<bool> {
        validate_key(key)?;
        validate_value(value)?;

        let mut data = self
            .data
            .write()
            .map_err(|_| StorageError::Internal("Failed to acquire write lock".to_string()))?;

        let stored_value = Value::new(value.to_string());
        let was_new = data.insert(key.to_string(), stored_value).is_none();

        self.put_ops.fetch_add(1, Ordering::Relaxed);
        Ok(was_new)
    }

    fn get(&self, key: &str) -> StorageResult<Value> {
        validate_key(key)?;

        let data = self
            .data
            .read()
            .map_err(|_| StorageError::Internal("Failed to acquire read lock".to_string()))?;

        self.get_ops.fetch_add(1, Ordering::Relaxed);

        data.get(key)
            .cloned()
            .ok_or_else(|| StorageError::KeyNotFound(key.to_string()))
    }

    fn delete(&self, key: &str) -> StorageResult<bool> {
        validate_key(key)?;

        let mut data = self
            .data
            .write()
            .map_err(|_| StorageError::Internal("Failed to acquire write lock".to_string()))?;

        self.delete_ops.fetch_add(1, Ordering::Relaxed);
        Ok(data.remove(key).is_some())
    }

    fn exists(&self, key: &str) -> StorageResult<bool> {
        validate_key(key)?;

        let data = self
            .data
            .read()
            .map_err(|_| StorageError::Internal("Failed to acquire read lock".to_string()))?;

        Ok(data.contains_key(key))
    }

    fn keys(&self) -> StorageResult<Vec<String>> {
        let data = self
            .data
            .read()
            .map_err(|_| StorageError::Internal("Failed to acquire read lock".to_string()))?;

        Ok(data.keys().cloned().collect())
    }

    fn values(&self) -> StorageResult<Vec<Value>> {
        let data = self
            .data
            .read()
            .map_err(|_| StorageError::Internal("Failed to acquire read lock".to_string()))?;

        Ok(data.values().cloned().collect())
    }

    fn all(&self) -> StorageResult<HashMap<String, Value>> {
        let data = self
            .data
            .read()
            .map_err(|_| StorageError::Internal("Failed to acquire read lock".to_string()))?;

        Ok(data.clone())
    }

    fn clear(&self) -> StorageResult<()> {
        let mut data = self
            .data
            .write()
            .map_err(|_| StorageError::Internal("Failed to acquire write lock".to_string()))?;

        data.clear();
        Ok(())
    }

    fn stats(&self) -> StorageResult<Stats> {
        let data = self
            .data
            .read()
            .map_err(|_| StorageError::Internal("Failed to acquire read lock".to_string()))?;

        Ok(Stats {
            key_count: data.len(),
            memory_usage: Self::calculate_memory_usage(&data),
            get_operations_count: self.get_ops.load(Ordering::Relaxed),
            put_operations_count: self.put_ops.load(Ordering::Relaxed),
            delete_operations_count: self.delete_ops.load(Ordering::Relaxed),
        })
    }

    fn size_of_value(&self, key: &str) -> StorageResult<usize> {
        validate_key(key)?;

        let data = self
            .data
            .read()
            .map_err(|_| StorageError::Internal("Failed to acquire read lock".to_string()))?;

        data.get(key)
            .map(|stored_value| stored_value.metadata.size)
            .ok_or_else(|| StorageError::KeyNotFound(key.to_string()))
    }
}

impl Clone for MemoryStorage {
    fn clone(&self) -> Self {
        Self {
            data: Arc::clone(&self.data),
            get_ops: AtomicU64::new(self.get_ops.load(Ordering::Relaxed)),
            put_ops: AtomicU64::new(self.put_ops.load(Ordering::Relaxed)),
            delete_ops: AtomicU64::new(self.delete_ops.load(Ordering::Relaxed)),
        }
    }
}

// Thread-safety: Arc<RwLock<_>> is Send + Sync, and AtomicU64 is Send + Sync
// unsafe impl Send for MemoryStorage {}
// unsafe impl Sync for MemoryStorage {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_storage() {
        let storage = MemoryStorage::new();
        let stats = storage.stats().unwrap();
        assert_eq!(stats.key_count, 0);
        assert_eq!(stats.memory_usage, 0);
    }

    #[test]
    fn test_put_and_get() {
        let storage = MemoryStorage::new();

        // Test putting a new key
        let was_new = storage.put("test_key", "test_value").unwrap();
        assert!(was_new);

        // Test getting the key
        let stored_value = storage.get("test_key").unwrap();
        assert_eq!(stored_value.value, "test_value");

        // Test updating existing key
        let was_new = storage.put("test_key", "updated_value").unwrap();
        assert!(!was_new);

        let stored_value = storage.get("test_key").unwrap();
        assert_eq!(stored_value.value, "updated_value");
    }

    #[test]
    fn test_delete() {
        let storage = MemoryStorage::new();

        storage.put("test_key", "test_value").unwrap();
        assert!(storage.exists("test_key").unwrap());

        let existed = storage.delete("test_key").unwrap();
        assert!(existed);
        assert!(!storage.exists("test_key").unwrap());

        // Try deleting non-existent key
        let existed = storage.delete("non_existent").unwrap();
        assert!(!existed);
    }

    #[test]
    fn test_list_operations() {
        let storage = MemoryStorage::new();

        storage.put("key1", "value1").unwrap();
        storage.put("key2", "value2").unwrap();
        storage.put("key3", "value3").unwrap();

        let keys = storage.keys().unwrap();
        assert_eq!(keys.len(), 3);
        assert!(keys.contains(&"key1".to_string()));
        assert!(keys.contains(&"key2".to_string()));
        assert!(keys.contains(&"key3".to_string()));

        let all_data = storage.all().unwrap();
        assert_eq!(all_data.len(), 3);
        assert_eq!(all_data.get("key1").unwrap().value, "value1");
    }

    #[test]
    fn test_clear() {
        let storage = MemoryStorage::new();

        storage.put("key1", "value1").unwrap();
        storage.put("key2", "value2").unwrap();

        assert_eq!(storage.stats().unwrap().key_count, 2);

        storage.clear().unwrap();
        assert_eq!(storage.stats().unwrap().key_count, 0);
    }

    #[test]
    fn test_stats() {
        let storage = MemoryStorage::new();

        storage.put("key1", "value1").unwrap();
        storage.get("key1").unwrap();
        storage.delete("key1").unwrap();

        let stats = storage.stats().unwrap();
        assert_eq!(stats.get_operations_count, 1);
        assert_eq!(stats.put_operations_count, 1);
        assert_eq!(stats.delete_operations_count, 1);
    }

    #[test]
    fn test_invalid_key() {
        let storage = MemoryStorage::new();

        // Empty key
        let result = storage.put("", "value");
        assert!(matches!(result, Err(StorageError::InvalidKey(_))));

        // Key with null byte
        let result = storage.put("key\0", "value");
        assert!(matches!(result, Err(StorageError::InvalidKey(_))));
    }

    #[test]
    fn test_key_not_found() {
        let storage = MemoryStorage::new();

        let result = storage.get("non_existent");
        assert!(matches!(result, Err(StorageError::KeyNotFound(_))));
    }
}

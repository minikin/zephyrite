use super::engine::{Stats, StorageEngine, Value};
use super::error::StorageResult;
use super::memory::MemoryStorage;
use super::wal::{WalManager, WalOperation};
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use tracing::{debug, info, warn};

/// Persistent storage engine that combines in-memory storage with Write-Ahead Logging
pub struct PersistentStorage {
    /// In-memory storage for fast access
    memory_storage: MemoryStorage,
    /// Write-Ahead Log manager for durability
    wal_manager: Arc<WalManager>,
}

impl PersistentStorage {
    /// Create a new persistent storage with WAL
    ///
    /// # Errors
    /// Returns an error if the WAL file cannot be created or accessed.
    pub fn new(wal_file_path: impl AsRef<Path>) -> StorageResult<Self> {
        let wal_manager = Arc::new(WalManager::new(wal_file_path)?);
        let memory_storage = MemoryStorage::new();

        let mut storage = Self {
            memory_storage,
            wal_manager,
        };

        storage.recover_from_wal()?;

        Ok(storage)
    }

    /// Create a new persistent storage with custom capacity and WAL settings
    ///
    /// # Errors
    /// Returns an error if the WAL file cannot be created or accessed.
    pub fn new_with_options(
        wal_file_path: impl AsRef<Path>,
        memory_capacity: usize,
        use_checksums: bool,
    ) -> StorageResult<Self> {
        let wal_manager = Arc::new(WalManager::new_with_options(wal_file_path, use_checksums)?);
        let memory_storage = MemoryStorage::with_capacity(memory_capacity);

        let mut storage = Self {
            memory_storage,
            wal_manager,
        };

        storage.recover_from_wal()?;

        Ok(storage)
    }

    /// Recover data from the Write-Ahead Log
    fn recover_from_wal(&mut self) -> StorageResult<()> {
        info!("Starting WAL recovery...");

        let entries = self.wal_manager.read_all_entries()?;

        if entries.is_empty() {
            info!("No WAL entries found, starting with empty storage");
            return Ok(());
        }

        info!("Recovering {} entries from WAL", entries.len());

        let mut recovered_ops = 0;
        let mut failed_ops = 0;

        self.recover(&entries, &mut recovered_ops, &mut failed_ops);

        if failed_ops > 0 {
            warn!(
                "WAL recovery completed with {} failed operations out of {} total",
                failed_ops,
                entries.len()
            );
        } else {
            info!(
                "WAL recovery completed successfully: {} operations recovered",
                recovered_ops
            );
        }

        Ok(())
    }

    fn recover(
        &mut self,
        entries: &Vec<super::wal::WalEntry>,
        recovered_ops: &mut i32,
        failed_ops: &mut i32,
    ) {
        for entry in entries {
            match &entry.operation {
                WalOperation::Put { key, value } => match self.memory_storage.put(key, value) {
                    Ok(_) => {
                        *recovered_ops += 1;
                        debug!("Recovered PUT operation: key={}", key);
                    }
                    Err(e) => {
                        *failed_ops += 1;
                        warn!("Failed to recover PUT operation for key '{}': {}", key, e);
                    }
                },
                WalOperation::Delete { key } => match self.memory_storage.delete(key) {
                    Ok(_) => {
                        *recovered_ops += 1;
                        debug!("Recovered DELETE operation: key={}", key);
                    }
                    Err(e) => {
                        *failed_ops += 1;
                        warn!(
                            "Failed to recover DELETE operation for key '{}': {}",
                            key, e
                        );
                    }
                },
                WalOperation::Clear => match self.memory_storage.clear() {
                    Ok(()) => {
                        *recovered_ops += 1;
                        debug!("Recovered CLEAR operation");
                    }
                    Err(e) => {
                        *failed_ops += 1;
                        warn!("Failed to recover CLEAR operation: {}", e);
                    }
                },
            }
        }
    }

    /// Get statistics including WAL information
    ///
    /// # Errors
    /// Returns an error if the storage statistics cannot be retrieved.
    pub fn detailed_stats(&self) -> StorageResult<DetailedStats> {
        let memory_stats = self.memory_storage.stats()?;
        let wal_sequence = self.wal_manager.current_sequence_number()?;

        Ok(DetailedStats {
            memory_stats,
            wal_file_path: self.wal_manager.file_pat().to_string(),
            wal_sequence_number: wal_sequence,
        })
    }

    /// Compact the WAL by creating a snapshot and truncating the log
    /// WARNING: This is a potentially dangerous operation that should be used carefully
    ///
    /// # Errors
    /// Returns an error if the WAL compaction fails or if storage operations fail.
    pub fn compact_wal(&self) -> StorageResult<CompactionResult> {
        info!("Starting WAL compaction...");

        let all_data = self.memory_storage.all()?;
        let entries_before = self.wal_manager.read_all_entries()?.len();

        // Truncate the WAL
        self.wal_manager.truncate()?;

        // Re-write all current data to the WAL
        let mut rewritten_entries = 0;
        for (key, value) in all_data {
            self.wal_manager.log_operation(WalOperation::Put {
                key,
                value: value.value,
            })?;
            rewritten_entries += 1;
        }

        info!(
            "WAL compaction completed: {} entries before, {} entries after",
            entries_before, rewritten_entries
        );

        Ok(CompactionResult {
            entries_before,
            entries_after: rewritten_entries,
        })
    }

    /// Get the path to the WAL file
    pub fn wal_file_path(&self) -> &str {
        self.wal_manager.file_pat()
    }
}

impl StorageEngine for PersistentStorage {
    fn put(&self, key: &str, value: &str) -> StorageResult<bool> {
        self.wal_manager.log_operation(WalOperation::Put {
            key: key.to_string(),
            value: value.to_string(),
        })?;

        self.memory_storage.put(key, value)
    }

    fn get(&self, key: &str) -> StorageResult<Value> {
        self.memory_storage.get(key)
    }

    fn delete(&self, key: &str) -> StorageResult<bool> {
        self.wal_manager.log_operation(WalOperation::Delete {
            key: key.to_string(),
        })?;

        self.memory_storage.delete(key)
    }

    fn exists(&self, key: &str) -> StorageResult<bool> {
        self.memory_storage.exists(key)
    }

    fn keys(&self) -> StorageResult<Vec<String>> {
        self.memory_storage.keys()
    }

    fn values(&self) -> StorageResult<Vec<Value>> {
        self.memory_storage.values()
    }

    fn all(&self) -> StorageResult<HashMap<String, Value>> {
        self.memory_storage.all()
    }

    fn clear(&self) -> StorageResult<()> {
        self.wal_manager.log_operation(WalOperation::Clear)?;

        self.memory_storage.clear()
    }

    fn stats(&self) -> StorageResult<Stats> {
        self.memory_storage.stats()
    }

    fn size_of_value(&self, key: &str) -> StorageResult<usize> {
        self.memory_storage.size_of_value(key)
    }
}

/// Detailed statistics including WAL information
#[derive(Debug, Clone, PartialEq)]
pub struct DetailedStats {
    /// Standard memory storage statistics
    pub memory_stats: Stats,
    /// Path to the WAL file
    pub wal_file_path: String,
    /// Current WAL sequence number
    pub wal_sequence_number: u64,
}

/// Result of a WAL compaction operation
#[derive(Debug, Clone, PartialEq)]
pub struct CompactionResult {
    /// Number of entries in WAL before compaction
    pub entries_before: usize,
    /// Number of entries in WAL after compaction
    pub entries_after: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_persistent_storage_basic_operations() {
        let temp_file = NamedTempFile::new().unwrap();
        let storage = PersistentStorage::new(temp_file.path()).unwrap();

        // Test put operation
        let was_new = storage.put("test_key", "test_value").unwrap();
        assert!(was_new);

        // Test get operation
        let retrieved = storage.get("test_key").unwrap();
        assert_eq!(retrieved.value, "test_value");

        // Test exists operation
        assert!(storage.exists("test_key").unwrap());
        assert!(!storage.exists("nonexistent").unwrap());

        // Test delete operation
        let was_deleted = storage.delete("test_key").unwrap();
        assert!(was_deleted);
        assert!(!storage.exists("test_key").unwrap());
    }

    #[test]
    fn test_persistent_storage_recovery() {
        let temp_file = NamedTempFile::new().unwrap();
        let temp_path = temp_file.path().to_path_buf();

        // Create storage, add some data, then drop it
        {
            let storage = PersistentStorage::new(&temp_path).unwrap();
            storage.put("key1", "value1").unwrap();
            storage.put("key2", "value2").unwrap();
            storage.delete("key1").unwrap();
        }

        // Create new storage instance with same WAL file
        let recovered_storage = PersistentStorage::new(&temp_path).unwrap();

        // Verify data was recovered correctly
        assert!(!recovered_storage.exists("key1").unwrap()); // Should be deleted
        assert!(recovered_storage.exists("key2").unwrap());

        let retrieved = recovered_storage.get("key2").unwrap();
        assert_eq!(retrieved.value, "value2");
    }

    #[test]
    fn test_persistent_storage_clear_operation() {
        let temp_file = NamedTempFile::new().unwrap();
        let storage = PersistentStorage::new(temp_file.path()).unwrap();

        // Add some data
        storage.put("key1", "value1").unwrap();
        storage.put("key2", "value2").unwrap();

        // Verify data exists
        assert_eq!(storage.keys().unwrap().len(), 2);

        // Clear all data
        storage.clear().unwrap();

        // Verify data is gone
        assert_eq!(storage.keys().unwrap().len(), 0);
        assert!(!storage.exists("key1").unwrap());
        assert!(!storage.exists("key2").unwrap());
    }

    #[test]
    fn test_persistent_storage_detailed_stats() {
        let temp_file = NamedTempFile::new().unwrap();
        let storage = PersistentStorage::new(temp_file.path()).unwrap();

        // Add some data to get meaningful stats
        storage.put("test", "value").unwrap();

        let detailed_stats = storage.detailed_stats().unwrap();
        assert_eq!(detailed_stats.memory_stats.key_count, 1);
        assert!(detailed_stats.wal_sequence_number > 0);
        assert!(!detailed_stats.wal_file_path.is_empty());
    }

    #[test]
    fn test_persistent_storage_compaction() {
        let temp_file = NamedTempFile::new().unwrap();
        let storage = PersistentStorage::new(temp_file.path()).unwrap();

        // Perform many operations to create WAL entries
        storage.put("key1", "value1").unwrap();
        storage.put("key2", "value2").unwrap();
        storage.delete("key1").unwrap();
        storage.put("key3", "value3").unwrap();
        storage.put("key2", "updated_value2").unwrap(); // Update existing key

        // Before compaction, we should have 5 WAL entries
        let entries_before = storage.wal_manager.read_all_entries().unwrap().len();
        assert_eq!(entries_before, 5);

        // Perform compaction
        let compaction_result = storage.compact_wal().unwrap();
        assert_eq!(compaction_result.entries_before, 5);
        assert_eq!(compaction_result.entries_after, 2); // Only key2 and key3 remain

        // Verify data integrity after compaction
        assert!(!storage.exists("key1").unwrap()); // Should still be deleted
        assert!(storage.exists("key2").unwrap());
        assert!(storage.exists("key3").unwrap());

        let key2_value = storage.get("key2").unwrap();
        assert_eq!(key2_value.value, "updated_value2");

        let key3_value = storage.get("key3").unwrap();
        assert_eq!(key3_value.value, "value3");
    }

    #[test]
    fn test_persistent_storage_recovery_after_compaction() {
        let temp_file = NamedTempFile::new().unwrap();
        let temp_path = temp_file.path().to_path_buf();

        // Create storage, add data, compact, then drop it
        {
            let storage = PersistentStorage::new(&temp_path).unwrap();
            storage.put("key1", "value1").unwrap();
            storage.put("key2", "value2").unwrap();
            storage.delete("key1").unwrap();
            storage.compact_wal().unwrap();
        }

        // Create new storage instance and verify recovery works after compaction
        let recovered_storage = PersistentStorage::new(&temp_path).unwrap();

        assert!(!recovered_storage.exists("key1").unwrap());
        assert!(recovered_storage.exists("key2").unwrap());

        let retrieved = recovered_storage.get("key2").unwrap();
        assert_eq!(retrieved.value, "value2");
    }
}

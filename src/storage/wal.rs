use super::error::{StorageError, StorageResult};
use crate::utils::time;
use serde::{Deserialize, Serialize};
use std::collections::hash_map::DefaultHasher;
use std::fs::{File, OpenOptions};
use std::hash::{Hash, Hasher};
use std::io::{BufRead, BufReader, Write};
use std::path::Path;
use std::sync::{Arc, Mutex};

/// Types of operations that can be logged in the WAL
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum WalOperation {
    /// Put operation: key, value
    Put {
        /// The key to store
        key: String,
        /// The value to store
        value: String,
    },
    /// Delete operation: key
    Delete {
        /// The key to delete
        key: String,
    },
    /// Clear operation: clear all data
    Clear,
}

/// A single entry in the Write-Ahead Log
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WalEntry {
    /// Unique sequence number for this entry
    pub sequence_number: u64,
    /// The operation being logged
    pub operation: WalOperation,
    /// Timestamp when the operation was logged
    pub timestamp: String,
    /// Optional checksum for integrity verification
    pub checksum: Option<String>,
}

impl WalEntry {
    /// Create a new WAL entry
    #[must_use]
    pub fn new(sequence_number: u64, operation: WalOperation) -> Self {
        Self {
            sequence_number,
            operation,
            timestamp: time::current_timestamp(),
            checksum: None,
        }
    }

    /// Create a new WAL entry with checksum
    #[must_use]
    pub fn new_with_checksum(sequence_number: u64, operation: WalOperation) -> Self {
        let mut entry = Self::new(sequence_number, operation);
        entry.checksum = Some(entry.calculate_checksum());
        entry
    }

    /// Calculate a simple checksum for the entry
    fn calculate_checksum(&self) -> String {
        let mut hasher = DefaultHasher::new();
        self.sequence_number.hash(&mut hasher);
        self.operation.hash(&mut hasher);
        self.timestamp.hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }

    /// Verify the checksum of the entry
    #[must_use]
    pub fn verify_checksum(&self) -> bool {
        match &self.checksum {
            Some(stored_checksum) => {
                let calculated_checksum = self.calculate_checksum();
                stored_checksum == &calculated_checksum
            }
            None => true, // No checksum to verify
        }
    }

    /// Serialize the entry to JSON string
    ///
    /// # Errors
    ///
    /// Returns a `StorageError::Internal` if JSON serialization fails.
    pub fn to_json(&self) -> StorageResult<String> {
        serde_json::to_string(self)
            .map_err(|e| StorageError::Internal(format!("Failed to serialize WAL entry: {e}")))
    }

    /// Deserialize the entry from JSON string
    ///
    /// # Errors
    ///
    /// Returns a `StorageError::Internal` if JSON deserialization fails.
    pub fn from_json(json: &str) -> StorageResult<Self> {
        serde_json::from_str(json)
            .map_err(|e| StorageError::Internal(format!("Failed to deserialize WAL entry: {e}")))
    }
}

// Implement Hash for WalOperation to enable checksum calculation
impl std::hash::Hash for WalOperation {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            WalOperation::Put { key, value } => {
                "put".hash(state);
                key.hash(state);
                value.hash(state);
            }
            WalOperation::Delete { key } => {
                "delete".hash(state);
                key.hash(state);
            }
            WalOperation::Clear => {
                "clear".hash(state);
            }
        }
    }
}

/// Write-Ahead Log manager
pub struct WalManager {
    /// Path to the WAL file
    file_path: String,
    /// File handle for writing to the WAL
    file: Arc<Mutex<File>>,
    /// Current sequence number
    sequence_number: Arc<Mutex<u64>>,
    /// Whether to use checksums for entries
    use_checksums: bool,
}

impl WalManager {
    /// Create a new WAL manager
    ///
    /// # Errors
    ///
    /// Returns a `StorageError::Internal` if the WAL file cannot be opened or created.
    pub fn new(file_path: impl AsRef<Path>) -> StorageResult<Self> {
        let file_path = file_path.as_ref().to_string_lossy().to_string();

        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&file_path)
            .map_err(|e| StorageError::Internal(format!("Failed to open WAL file: {e}")))?;

        Ok(Self {
            file_path,
            file: Arc::new(Mutex::new(file)),
            sequence_number: Arc::new(Mutex::new(0)),
            use_checksums: true,
        })
    }

    /// Create a new WAL manager with custom settings
    ///
    /// # Errors
    ///
    /// Returns a `StorageError::Internal` if the WAL file cannot be opened or created.
    pub fn new_with_options(
        file_path: impl AsRef<Path>,
        use_checksums: bool,
    ) -> StorageResult<Self> {
        let mut manager = Self::new(file_path)?;
        manager.use_checksums = use_checksums;
        Ok(manager)
    }

    /// Write an operation to the WAL
    ///
    /// # Errors
    ///
    /// Returns a `StorageError::Internal` if:
    /// - The sequence number lock cannot be acquired
    /// - The file lock cannot be acquired
    /// - Writing to the WAL file fails
    /// - Flushing the WAL file fails
    /// - JSON serialization of the entry fails
    pub fn log_operation(&self, operation: WalOperation) -> StorageResult<u64> {
        let sequence_number = {
            let mut seq = self.sequence_number.lock().map_err(|_| {
                StorageError::Internal("Failed to acquire sequence number lock".to_string())
            })?;
            *seq += 1;
            *seq
        };

        let entry = if self.use_checksums {
            WalEntry::new_with_checksum(sequence_number, operation)
        } else {
            WalEntry::new(sequence_number, operation)
        };

        let json_line = entry.to_json()?;

        {
            let mut file = self
                .file
                .lock()
                .map_err(|_| StorageError::Internal("Failed to acquire file lock".to_string()))?;

            writeln!(file, "{json_line}")
                .map_err(|e| StorageError::Internal(format!("Failed to write to WAL: {e}")))?;

            file.flush()
                .map_err(|e| StorageError::Internal(format!("Failed to flush WAL: {e}")))?;
        }

        Ok(sequence_number)
    }

    /// Read all entries from the WAL file
    ///
    /// # Errors
    ///
    /// Returns a `StorageError::Internal` if:
    /// - The WAL file cannot be opened for reading
    /// - A line in the file cannot be read
    /// - JSON deserialization of an entry fails
    /// - Checksum verification fails for an entry
    /// - The sequence number lock cannot be acquired
    pub fn read_all_entries(&self) -> StorageResult<Vec<WalEntry>> {
        let file = File::open(&self.file_path).map_err(|e| {
            StorageError::Internal(format!("Failed to open WAL file for reading: {e}"))
        })?;

        let reader = BufReader::new(file);
        let mut entries = Vec::new();

        for (line_num, line) in reader.lines().enumerate() {
            let line = line.map_err(|e| {
                StorageError::Internal(format!(
                    "Failed to read line {} from WAL: {}",
                    line_num + 1,
                    e
                ))
            })?;

            if line.trim().is_empty() {
                continue;
            }

            let entry = WalEntry::from_json(&line)?;

            if !entry.verify_checksum() {
                return Err(StorageError::Internal(format!(
                    "Checksum verification failed for WAL entry at line {}",
                    line_num + 1
                )));
            }

            entries.push(entry);
        }

        // Update the sequence number to the highest seen
        if let Some(last_entry) = entries.last() {
            let mut seq = self.sequence_number.lock().map_err(|_| {
                StorageError::Internal("Failed to acquire sequence number lock".to_string())
            })?;
            *seq = last_entry.sequence_number;
        }

        Ok(entries)
    }

    /// Get the current sequence number
    ///
    /// # Errors
    ///
    /// Returns a `StorageError::Internal` if the sequence number lock cannot be acquired.
    pub fn current_sequence_number(&self) -> StorageResult<u64> {
        let seq = self.sequence_number.lock().map_err(|_| {
            StorageError::Internal("Failed to acquire sequence number lock".to_string())
        })?;
        Ok(*seq)
    }

    /// Truncate the WAL file (use with caution!)
    ///
    /// # Errors
    ///
    /// Returns a `StorageError::Internal` if:
    /// - The file lock cannot be acquired
    /// - Truncating the WAL file fails
    /// - Flushing the WAL file after truncate fails
    /// - The sequence number lock cannot be acquired
    pub fn truncate(&self) -> StorageResult<()> {
        {
            let mut file = self
                .file
                .lock()
                .map_err(|_| StorageError::Internal("Failed to acquire file lock".to_string()))?;

            file.set_len(0)
                .map_err(|e| StorageError::Internal(format!("Failed to truncate WAL file: {e}")))?;

            file.flush().map_err(|e| {
                StorageError::Internal(format!("Failed to flush WAL file after truncate: {e}"))
            })?;
        }

        // Reset sequence number
        {
            let mut seq = self.sequence_number.lock().map_err(|_| {
                StorageError::Internal("Failed to acquire sequence number lock".to_string())
            })?;
            *seq = 0;
        }

        Ok(())
    }

    /// Get the path to the WAL file
    #[must_use]
    pub fn file_pat(&self) -> &str {
        &self.file_path
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_wal_entry_creation() {
        let operation = WalOperation::Put {
            key: "test".to_string(),
            value: "value".to_string(),
        };
        let entry = WalEntry::new(1, operation.clone());

        assert_eq!(entry.sequence_number, 1);
        assert_eq!(entry.operation, operation);
        assert!(entry.checksum.is_none());
    }

    #[test]
    fn test_wal_entry_with_checksum() {
        let operation = WalOperation::Put {
            key: "test".to_string(),
            value: "value".to_string(),
        };
        let entry = WalEntry::new_with_checksum(1, operation.clone());

        assert_eq!(entry.sequence_number, 1);
        assert_eq!(entry.operation, operation);
        assert!(entry.checksum.is_some());
        assert!(entry.verify_checksum());
    }

    #[test]
    fn test_wal_entry_serialization() {
        let operation = WalOperation::Delete {
            key: "test".to_string(),
        };
        let entry = WalEntry::new_with_checksum(42, operation);

        let json = entry.to_json().unwrap();
        let deserialized = WalEntry::from_json(&json).unwrap();

        assert_eq!(entry, deserialized);
        assert!(deserialized.verify_checksum());
    }

    #[test]
    fn test_wal_manager_basic_operations() {
        let temp_file = NamedTempFile::new().unwrap();
        let wal_manager = WalManager::new(temp_file.path()).unwrap();

        // Test logging operations
        let seq1 = wal_manager
            .log_operation(WalOperation::Put {
                key: "key1".to_string(),
                value: "value1".to_string(),
            })
            .unwrap();

        let seq2 = wal_manager
            .log_operation(WalOperation::Delete {
                key: "key2".to_string(),
            })
            .unwrap();

        assert_eq!(seq1, 1);
        assert_eq!(seq2, 2);

        // Test reading entries
        let entries = wal_manager.read_all_entries().unwrap();
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].sequence_number, 1);
        assert_eq!(entries[1].sequence_number, 2);

        match &entries[0].operation {
            WalOperation::Put { key, value } => {
                assert_eq!(key, "key1");
                assert_eq!(value, "value1");
            }
            _ => panic!("Expected Put operation"),
        }

        match &entries[1].operation {
            WalOperation::Delete { key } => {
                assert_eq!(key, "key2");
            }
            _ => panic!("Expected Delete operation"),
        }
    }

    #[test]
    fn test_wal_manager_truncate() {
        let temp_file = NamedTempFile::new().unwrap();
        let wal_manager = WalManager::new(temp_file.path()).unwrap();

        wal_manager
            .log_operation(WalOperation::Put {
                key: "key1".to_string(),
                value: "value1".to_string(),
            })
            .unwrap();

        wal_manager.log_operation(WalOperation::Clear).unwrap();

        // Verify entries exist
        assert_eq!(wal_manager.read_all_entries().unwrap().len(), 2);

        wal_manager.truncate().unwrap();

        // Verify entries are gone
        assert_eq!(wal_manager.read_all_entries().unwrap().len(), 0);
        assert_eq!(wal_manager.current_sequence_number().unwrap(), 0);
    }
}

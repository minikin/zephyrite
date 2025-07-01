//! HTTP Server Configuration
use std::net::SocketAddr;
use std::path::PathBuf;

#[derive(Debug, Clone)]
/// Configurations for the application.
pub struct Config {
    /// Server bind address
    pub address: SocketAddr,
    /// Storage configuration options
    pub storage: StorageConfig,
}

#[derive(Debug, Clone)]
/// Storage configuration options
pub struct StorageConfig {
    /// Type of storage to use
    pub storage_type: StorageType,
    /// Path to the WAL file (for persistent storage)
    pub wal_file_path: Option<PathBuf>,
    /// Initial memory capacity for storage
    pub memory_capacity: Option<usize>,
    /// Whether to use checksums in WAL entries
    pub use_checksums: bool,
}

#[derive(Debug, Clone, PartialEq)]
/// Types of storage engines available
pub enum StorageType {
    /// In-memory only storage (no persistence)
    Memory,
    /// Persistent storage with WAL
    Persistent,
}

impl Config {
    /// Creates a new configuration with the specified port.
    ///
    /// # Arguments
    ///
    /// * `port` - The address to bind the server to.
    #[must_use]
    pub fn new(port: u16) -> Self {
        Self {
            address: SocketAddr::from(([127, 0, 0, 1], port)),
            storage: StorageConfig::default(),
        }
    }

    /// Creates a new configuration with persistent storage
    ///
    /// # Arguments
    ///
    /// * `port` - The address to bind the server to.
    /// * `wal_file_path` - Path to the WAL file for persistence
    #[must_use]
    pub fn new_with_persistence(port: u16, wal_file_path: PathBuf) -> Self {
        Self {
            address: SocketAddr::from(([127, 0, 0, 1], port)),
            storage: StorageConfig {
                storage_type: StorageType::Persistent,
                wal_file_path: Some(wal_file_path),
                memory_capacity: None,
                use_checksums: true,
            },
        }
    }

    /// Creates a new configuration with custom storage options
    ///
    /// # Arguments
    ///
    /// * `port` - The address to bind the server to.
    /// * `storage_config` - Custom storage configuration
    #[must_use]
    pub fn new_with_storage(port: u16, storage_config: StorageConfig) -> Self {
        Self {
            address: SocketAddr::from(([127, 0, 0, 1], port)),
            storage: storage_config,
        }
    }
}

impl Default for Config {
    /// Returns the default configuration with the address set to port 8080.
    fn default() -> Self {
        Self::new(8080)
    }
}

impl StorageConfig {
    /// Creates a new storage config for in-memory storage
    #[must_use]
    pub fn memory() -> Self {
        Self {
            storage_type: StorageType::Memory,
            wal_file_path: None,
            memory_capacity: None,
            use_checksums: false,
        }
    }

    /// Creates a new storage config for persistent storage
    #[must_use]
    pub fn persistent(wal_file_path: PathBuf) -> Self {
        Self {
            storage_type: StorageType::Persistent,
            wal_file_path: Some(wal_file_path),
            memory_capacity: None,
            use_checksums: true,
        }
    }

    /// Set the memory capacity for the storage
    #[must_use]
    pub fn with_memory_capacity(mut self, capacity: usize) -> Self {
        self.memory_capacity = Some(capacity);
        self
    }

    /// Set whether to use checksums in WAL entries
    #[must_use]
    pub fn with_checksums(mut self, use_checksums: bool) -> Self {
        self.use_checksums = use_checksums;
        self
    }
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self::memory()
    }
}

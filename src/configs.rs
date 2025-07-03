//! HTTP Server Configuration
use std::net::SocketAddr;

/// Storage backend type
#[derive(Debug, Clone)]
pub enum StorageType {
    /// In-memory storage
    Memory,
    /// Persistent storage with WAL
    Persistent,
}

/// Storage configuration
#[derive(Debug, Clone)]
pub struct StorageConfig {
    /// Type of storage backend to use
    pub storage_type: StorageType,
    /// Memory capacity limit (bytes)
    pub memory_capacity: Option<usize>,
    /// WAL file path for persistent storage
    pub wal_file_path: Option<String>,
    /// Whether to use checksums for data integrity
    pub use_checksums: bool,
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            storage_type: StorageType::Memory,
            memory_capacity: None,
            wal_file_path: None,
            use_checksums: true,
        }
    }
}

impl StorageConfig {
    /// Creates a new persistent storage configuration
    #[must_use]
    pub fn persistent(wal_file_path: impl Into<String>) -> Self {
        Self {
            storage_type: StorageType::Persistent,
            memory_capacity: None,
            wal_file_path: Some(wal_file_path.into()),
            use_checksums: true,
        }
    }

    /// Creates a new memory storage configuration
    #[must_use]
    pub fn memory() -> Self {
        Self {
            storage_type: StorageType::Memory,
            memory_capacity: None,
            wal_file_path: None,
            use_checksums: true,
        }
    }

    /// Sets memory capacity for the storage
    #[must_use]
    pub fn with_memory_capacity(mut self, capacity: usize) -> Self {
        self.memory_capacity = Some(capacity);
        self
    }

    /// Sets whether to use checksums
    #[must_use]
    pub fn with_checksums(mut self, use_checksums: bool) -> Self {
        self.use_checksums = use_checksums;
        self
    }
}

#[derive(Debug, Clone)]
/// Configurations for the application.
pub struct Config {
    /// Server bind address
    pub address: SocketAddr,
    /// Storage configuration
    pub storage: StorageConfig,
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

    /// Creates a new configuration with storage settings.
    ///
    /// # Arguments
    ///
    /// * `port` - The port to bind the server to
    /// * `storage` - Storage configuration
    #[must_use]
    pub fn with_storage(port: u16, storage: StorageConfig) -> Self {
        Self {
            address: SocketAddr::from(([127, 0, 0, 1], port)),
            storage,
        }
    }
}

impl Default for Config {
    /// Returns the default configuration with the address set to port 8080.
    fn default() -> Self {
        Self::new(8080)
    }
}

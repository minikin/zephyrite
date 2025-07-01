//! # Zephyrite
//!
//! A high-performance key-value store.

/// Current version of Zephyrite
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

pub mod configs;
pub mod server;
pub mod storage;
/// Utility functions and helpers
pub mod utils;

pub use configs::{Config, StorageConfig, StorageType};
pub use server::Server;
pub use storage::{MemoryStorage, PersistentStorage, StorageEngine, StorageError, StorageResult};

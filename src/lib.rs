//! # Zephyrite
//!
//! A high-performance key-value store.

/// Current version of Zephyrite
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

pub mod configs;
pub mod server;

pub use configs::Config;
pub use server::Server;

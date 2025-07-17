//! Disk-based storage implementation
//!
//! This module provides disk-based storage functionality including:
//! - Page management for efficient disk storage
//! - File header management for database files

pub mod buffer;
pub mod header;
pub mod index;
pub mod page;
/// Page manager for handling disk-based page operations
pub mod page_manager;

pub use page::Page;
pub use page_manager::PageManager;

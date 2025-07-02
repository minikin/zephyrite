//! Disk-based storage engine with page management

use crate::{StorageError, StorageResult};

/// Page size for disk storage (4KB)
const PAGE_SIZE: u16 = 4096;

/// Version of the storage format
const FORMAT_VERSION: u16 = 1;

/// Number to identify Zephyrite database files
const ZEPHYRITE: [u8; 9] = *b"ZEPHYRITE";

/// Header for the database file
#[repr(C)]
#[derive(Debug, Clone)]
struct FileHeader {
    zephyrite_file_id: [u8; 9],
    version: u16,
    page_size: u16,
    next_page: u64,
    free_pages_count: u64,
    index_page_id: u64,
}
#[allow(dead_code)]
impl FileHeader {
    fn new() -> Self {
        Self {
            zephyrite_file_id: ZEPHYRITE,
            version: FORMAT_VERSION,
            page_size: PAGE_SIZE,
            next_page: 1,
            free_pages_count: 0,
            index_page_id: 0,
        }
    }

    fn serialize(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(64);
        bytes.extend_from_slice(&self.zephyrite_file_id);
        bytes.extend_from_slice(&self.version.to_le_bytes());
        bytes.extend_from_slice(&self.page_size.to_le_bytes());
        bytes.extend_from_slice(&self.next_page.to_le_bytes());
        bytes.extend_from_slice(&self.free_pages_count.to_le_bytes());
        bytes.extend_from_slice(&self.index_page_id.to_le_bytes());
        bytes.resize(64, 0);
        bytes
    }

    fn deserialize(bytes: &[u8]) -> StorageResult<Self> {
        if bytes.len() < 64 {
            return Err(StorageError::Internal("Invalid header size".to_string()));
        }

        if bytes[0..9] != ZEPHYRITE {
            return Err(StorageError::Internal(
                "Invalid Zephyrite file identifier".to_string(),
            ));
        }

        let mut zephyrite_file_id = [0u8; 9];
        zephyrite_file_id.copy_from_slice(&bytes[0..9]);
        let version = u16::from_le_bytes([bytes[9], bytes[10]]);
        let page_size = u16::from_le_bytes([bytes[11], bytes[12]]);
        let next_page = u64::from_le_bytes([
            bytes[13], bytes[14], bytes[15], bytes[16], bytes[17], bytes[18], bytes[19], bytes[20],
        ]);
        let free_pages_count = u64::from_le_bytes([
            bytes[21], bytes[22], bytes[23], bytes[24], bytes[25], bytes[26], bytes[27], bytes[28],
        ]);
        let index_page_id = u64::from_le_bytes([
            bytes[29], bytes[30], bytes[31], bytes[32], bytes[33], bytes[34], bytes[35], bytes[36],
        ]);

        Ok(Self {
            zephyrite_file_id,
            version,
            page_size,
            next_page,
            free_pages_count,
            index_page_id,
        })
    }
}

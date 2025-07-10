//! Disk-based storage engine with page management

use crate::{StorageError, StorageResult};

/// Page size for disk storage (4KB)
pub const PAGE_SIZE: u16 = 4096;

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
    /// Size of the serialized header in bytes
    const HEADER_SIZE: usize = 64;

    pub fn new() -> Self {
        Self {
            zephyrite_file_id: ZEPHYRITE,
            version: FORMAT_VERSION,
            page_size: PAGE_SIZE,
            next_page: 1,
            free_pages_count: 0,
            index_page_id: 0,
        }
    }

    pub fn serialize(&self) -> StorageResult<[u8; Self::HEADER_SIZE]> {
        const EXPECTED_DATA_SIZE: usize = 9 + 2 + 2 + 8 + 8 + 8; // 37 bytes
        let mut bytes = [0u8; Self::HEADER_SIZE];
        let mut offset = 0;

        offset = Self::write_bytes_at(&mut bytes, offset, &self.zephyrite_file_id)?;
        offset = Self::write_bytes_at(&mut bytes, offset, &self.version.to_le_bytes())?;
        offset = Self::write_bytes_at(&mut bytes, offset, &self.page_size.to_le_bytes())?;
        offset = Self::write_bytes_at(&mut bytes, offset, &self.next_page.to_le_bytes())?;
        offset = Self::write_bytes_at(&mut bytes, offset, &self.free_pages_count.to_le_bytes())?;
        let final_offset =
            Self::write_bytes_at(&mut bytes, offset, &self.index_page_id.to_le_bytes())?;

        if final_offset != EXPECTED_DATA_SIZE {
            return Err(StorageError::Internal(format!(
                "Serialization size mismatch: expected {EXPECTED_DATA_SIZE}, got {final_offset}"
            )));
        }

        Ok(bytes)
    }

    /// Serialize method returning Vec<u8>
    pub fn serialize_vec(&self) -> StorageResult<Vec<u8>> {
        Ok(self.serialize()?.to_vec())
    }

    /// Deserialize header from byte slice
    pub fn deserialize(bytes: &[u8]) -> StorageResult<Self> {
        if bytes.len() < Self::HEADER_SIZE {
            return Err(StorageError::Internal(format!(
                "Invalid header size: expected {}, got {}",
                Self::HEADER_SIZE,
                bytes.len()
            )));
        }

        if bytes[0..9] != ZEPHYRITE {
            return Err(StorageError::Internal(
                "Invalid Zephyrite file identifier".to_string(),
            ));
        }

        let mut zephyrite_file_id = [0u8; 9];
        zephyrite_file_id.copy_from_slice(&bytes[0..9]);

        let version = Self::read_u16_le(bytes, 9)?;
        let page_size = Self::read_u16_le(bytes, 11)?;
        let next_page = Self::read_u64_le(bytes, 13)?;
        let free_pages_count = Self::read_u64_le(bytes, 21)?;
        let index_page_id = Self::read_u64_le(bytes, 29)?;

        let header = Self {
            zephyrite_file_id,
            version,
            page_size,
            next_page,
            free_pages_count,
            index_page_id,
        };

        header.validate()?;
        Ok(header)
    }

    /// Helper function to safely read a u16 from bytes at given offset
    fn read_u16_le(bytes: &[u8], offset: usize) -> StorageResult<u16> {
        if offset + 2 > bytes.len() {
            return Err(StorageError::Internal(format!(
                "Buffer too short for u16 at offset {} : need {}, have {}",
                offset,
                offset + 2,
                bytes.len()
            )));
        }

        Ok(u16::from_le_bytes([bytes[offset], bytes[offset + 1]]))
    }

    /// Helper function to safely read a u64 from bytes at given offset
    fn read_u64_le(bytes: &[u8], offset: usize) -> StorageResult<u64> {
        if offset + 8 > bytes.len() {
            return Err(StorageError::Internal(format!(
                "Buffer too short for u64 at offset {}: need {}, have {}",
                offset,
                offset + 8,
                bytes.len()
            )));
        }

        Ok(u64::from_le_bytes([
            bytes[offset],
            bytes[offset + 1],
            bytes[offset + 2],
            bytes[offset + 3],
            bytes[offset + 4],
            bytes[offset + 5],
            bytes[offset + 6],
            bytes[offset + 7],
        ]))
    }

    /// Helper function to safely write bytes to buffer at given offset
    /// Returns the new offset after writing
    fn write_bytes_at(buffer: &mut [u8], offset: usize, data: &[u8]) -> StorageResult<usize> {
        if offset + data.len() > buffer.len() {
            return Err(StorageError::Internal(format!(
                "Buffer too small for write at offset {}: need {}, have {}",
                offset,
                offset + data.len(),
                buffer.len()
            )));
        }

        buffer[offset..offset + data.len()].copy_from_slice(data);
        Ok(offset + data.len())
    }

    /// Validate header fields
    fn validate(&self) -> StorageResult<()> {
        if self.version == 0 || self.version > FORMAT_VERSION {
            return Err(StorageError::Internal(format!(
                "Unsupported format version: {}",
                self.version
            )));
        }

        if self.page_size == 0 || (self.page_size & (self.page_size - 1)) != 0 {
            return Err(StorageError::Internal(format!(
                "Invalid page size: {} (must be power of 2)",
                self.page_size
            )));
        }

        if self.next_page == 0 {
            return Err(StorageError::Internal(
                "Invalid next_page: cannot be 0".to_string(),
            ));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_header_new() {
        let header = FileHeader::new();

        assert_eq!(header.zephyrite_file_id, ZEPHYRITE);
        assert_eq!(header.version, FORMAT_VERSION);
        assert_eq!(header.page_size, PAGE_SIZE);
        assert_eq!(header.next_page, 1);
        assert_eq!(header.free_pages_count, 0);
        assert_eq!(header.index_page_id, 0);
    }

    #[test]
    fn test_file_header_serialize() {
        let header = FileHeader::new();
        let serialized = header.serialize().unwrap();

        assert_eq!(serialized.len(), FileHeader::HEADER_SIZE);

        // Check that the first 9 bytes contain the ZEPHYRITE identifier
        assert_eq!(&serialized[0..9], &ZEPHYRITE);

        // Check that the rest of the buffer is properly filled
        assert_ne!(serialized, [0u8; FileHeader::HEADER_SIZE]);
    }

    #[test]
    fn test_file_header_serialize_vec() {
        let header = FileHeader::new();
        let serialized = header.serialize_vec().unwrap();

        assert_eq!(serialized.len(), FileHeader::HEADER_SIZE);
        assert_eq!(&serialized[0..9], &ZEPHYRITE);
    }

    #[test]
    fn test_file_header_serialize_deserialize_roundtrip() {
        let original = FileHeader::new();
        let serialized = original.serialize().unwrap();
        let deserialized = FileHeader::deserialize(&serialized).unwrap();

        assert_eq!(original.zephyrite_file_id, deserialized.zephyrite_file_id);
        assert_eq!(original.version, deserialized.version);
        assert_eq!(original.page_size, deserialized.page_size);
        assert_eq!(original.next_page, deserialized.next_page);
        assert_eq!(original.free_pages_count, deserialized.free_pages_count);
        assert_eq!(original.index_page_id, deserialized.index_page_id);
    }

    #[test]
    fn test_file_header_deserialize_invalid_size() {
        let short_bytes = vec![0u8; FileHeader::HEADER_SIZE - 1];
        let result = FileHeader::deserialize(&short_bytes);

        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Invalid header size")
        );
    }

    #[test]
    fn test_file_header_deserialize_invalid_identifier() {
        let mut bytes = [0u8; FileHeader::HEADER_SIZE];
        bytes[0..9].copy_from_slice(b"WRONGFILE");

        let result = FileHeader::deserialize(&bytes);

        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Invalid Zephyrite file identifier")
        );
    }

    #[test]
    fn test_file_header_validate_invalid_version() {
        let mut header = FileHeader::new();
        header.version = 0;

        let result = header.validate();
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Unsupported format version")
        );

        header.version = FORMAT_VERSION + 1;
        let result = header.validate();
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Unsupported format version")
        );
    }

    #[test]
    fn test_file_header_validate_invalid_page_size() {
        let mut header = FileHeader::new();

        // Test page size of 0
        header.page_size = 0;
        let result = header.validate();
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Invalid page size")
        );

        // Test non-power-of-2 page size
        header.page_size = 1000; // Not a power of 2
        let result = header.validate();
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Invalid page size")
        );

        // Test valid power-of-2 page sizes
        for page_size in [512, 1024, 2048, 4096, 8192] {
            header.page_size = page_size;
            assert!(header.validate().is_ok());
        }
    }

    #[test]
    fn test_file_header_validate_invalid_next_page() {
        let mut header = FileHeader::new();
        header.next_page = 0;

        let result = header.validate();
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Invalid next_page: cannot be 0")
        );
    }

    #[test]
    fn test_file_header_read_u16_le() {
        let bytes = [0x34, 0x12, 0x78, 0x56]; // Little-endian representation

        let value = FileHeader::read_u16_le(&bytes, 0).unwrap();
        assert_eq!(value, 0x1234);

        let value = FileHeader::read_u16_le(&bytes, 2).unwrap();
        assert_eq!(value, 0x5678);

        // Test out of bounds
        let result = FileHeader::read_u16_le(&bytes, 3);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Buffer too short for u16")
        );
    }

    #[test]
    fn test_file_header_read_u64_le() {
        let bytes = [0x78, 0x56, 0x34, 0x12, 0xBC, 0x9A, 0x78, 0x56, 0xFF]; // Little-endian representation

        let value = FileHeader::read_u64_le(&bytes, 0).unwrap();
        assert_eq!(value, 0x5678_9ABC_1234_5678);

        // Test out of bounds
        let result = FileHeader::read_u64_le(&bytes, 2);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Buffer too short for u64")
        );
    }

    #[test]
    fn test_file_header_write_bytes_at() {
        let mut buffer = [0u8; 10];
        let data = [0x12, 0x34, 0x56, 0x78];

        let new_offset = FileHeader::write_bytes_at(&mut buffer, 2, &data).unwrap();
        assert_eq!(new_offset, 6);
        assert_eq!(&buffer[2..6], &data);

        // Test out of bounds
        let result = FileHeader::write_bytes_at(&mut buffer, 8, &data);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Buffer too small for write")
        );
    }

    #[test]
    fn test_file_header_serialize_consistency() {
        let mut header = FileHeader::new();
        header.next_page = 42;
        header.free_pages_count = 100;
        header.index_page_id = 200;

        let serialized = header.serialize().unwrap();
        let deserialized = FileHeader::deserialize(&serialized).unwrap();

        // Verify all fields are correctly serialized and deserialized
        assert_eq!(header.zephyrite_file_id, deserialized.zephyrite_file_id);
        assert_eq!(header.version, deserialized.version);
        assert_eq!(header.page_size, deserialized.page_size);
        assert_eq!(header.next_page, deserialized.next_page);
        assert_eq!(header.free_pages_count, deserialized.free_pages_count);
        assert_eq!(header.index_page_id, deserialized.index_page_id);
    }

    #[test]
    fn test_file_header_constants() {
        assert_eq!(PAGE_SIZE, 4096);
        assert_eq!(FORMAT_VERSION, 1);
        assert_eq!(ZEPHYRITE, *b"ZEPHYRITE");
        assert_eq!(FileHeader::HEADER_SIZE, 64);
    }
}

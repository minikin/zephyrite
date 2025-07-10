//! Page management for disk storage
//!
//! Pages are the fundamental unit of storage in the disk-based engine.
//! Each page is a fixed-size block that can store data efficiently.

use super::header::PAGE_SIZE;

/// A page in the database file
///
/// Pages are fixed-size blocks that store data on disk. Each page has a unique ID
/// and can be marked as dirty when modified in memory.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Page {
    /// Unique identifier for the page
    pub id: u64,
    /// Raw data stored in the page
    pub data: Vec<u8>,
    /// Whether the page has been modified since it was last written to disk
    pub dirty: bool,
}

impl Page {
    /// Creates a new page with the given ID and with zeroed data.
    #[must_use]
    pub fn new(id: u64) -> Self {
        Self {
            id,
            data: vec![0; PAGE_SIZE as usize],
            dirty: false,
        }
    }

    /// Creates a new page from existing data.
    ///
    /// # Panics
    ///
    /// Panics if the data length exceeds the page size.
    #[must_use]
    pub fn from_data(id: u64, data: Vec<u8>) -> Self {
        assert!(data.len() <= PAGE_SIZE as usize, "Data exceeds page size");

        let mut data = data;
        // Ensure the data is exactly PAGE_SIZE bytes
        data.resize(PAGE_SIZE as usize, 0);

        Self {
            id,
            data,
            dirty: false,
        }
    }

    /// Mark the page as dirty (modified)
    pub fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    /// Check if the page is dirty
    #[must_use]
    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    /// Clear the dirty flag
    pub fn clear_dirty(&mut self) {
        self.dirty = false;
    }

    /// Get page size
    #[must_use]
    pub fn size(&self) -> usize {
        self.data.len()
    }

    /// Write data to the page at a specific offset
    ///
    /// # Errors
    ///
    /// Returns an error if the data would exceed the page size when written at the given offset.
    pub fn write_data(&mut self, offset: usize, data: &[u8]) -> Result<(), String> {
        if offset + data.len() > PAGE_SIZE as usize {
            return Err("Data exceeds page size".to_string());
        }

        self.data[offset..offset + data.len()].copy_from_slice(data);
        self.mark_dirty();
        Ok(())
    }

    /// Read data from the page at a specific offset
    ///
    /// # Errors
    ///
    /// Returns an error if the read would exceed the page size.
    pub fn read_data(&self, offset: usize, length: usize) -> Result<&[u8], String> {
        if offset + length > PAGE_SIZE as usize {
            return Err("Read exceeds page size".to_string());
        }

        Ok(&self.data[offset..offset + length])
    }

    /// Get the amount of free space in the page
    #[must_use]
    pub fn free_space(&self) -> usize {
        PAGE_SIZE as usize - self.data.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_page_new() {
        let page = Page::new(42);

        assert_eq!(page.id, 42);
        assert_eq!(page.data.len(), PAGE_SIZE as usize);
        assert!(!page.dirty);
        assert!(!page.is_dirty());
        assert_eq!(page.size(), PAGE_SIZE as usize);
        assert_eq!(page.free_space(), 0);

        // All data should be zeroed
        assert!(page.data.iter().all(|&x| x == 0));
    }

    #[test]
    fn test_page_from_data() {
        let data = vec![1, 2, 3, 4, 5];
        let page = Page::from_data(99, data.clone());

        assert_eq!(page.id, 99);
        assert_eq!(page.data.len(), PAGE_SIZE as usize);
        assert!(!page.dirty);
        assert!(!page.is_dirty());

        // First 5 bytes should match our data
        assert_eq!(&page.data[0..5], &data);

        // Rest should be zeros
        assert!(page.data[5..].iter().all(|&x| x == 0));
    }

    #[test]
    fn test_page_from_data_exact_size() {
        let data = vec![42; PAGE_SIZE as usize];
        let page = Page::from_data(1, data.clone());

        assert_eq!(page.id, 1);
        assert_eq!(page.data.len(), PAGE_SIZE as usize);
        assert_eq!(page.data, data);
        assert!(!page.dirty);
    }

    #[test]
    #[should_panic(expected = "Data exceeds page size")]
    fn test_page_from_data_too_large() {
        let data = vec![1; PAGE_SIZE as usize + 1];
        let _page = Page::from_data(1, data);
    }

    #[test]
    fn test_page_mark_dirty() {
        let mut page = Page::new(1);

        assert!(!page.is_dirty());

        page.mark_dirty();
        assert!(page.is_dirty());
        assert!(page.dirty);
    }

    #[test]
    fn test_page_clear_dirty() {
        let mut page = Page::new(1);

        page.mark_dirty();
        assert!(page.is_dirty());

        page.clear_dirty();
        assert!(!page.is_dirty());
        assert!(!page.dirty);
    }

    #[test]
    fn test_page_write_data() {
        let mut page = Page::new(1);
        let data = b"hello world";

        assert!(!page.is_dirty());

        let result = page.write_data(0, data);
        assert!(result.is_ok());
        assert!(page.is_dirty());

        assert_eq!(&page.data[0..data.len()], data);
        assert_eq!(page.data[data.len()], 0);
    }

    #[test]
    fn test_page_write_data_with_offset() {
        let mut page = Page::new(1);
        let data = b"test";
        let offset = 100;

        let result = page.write_data(offset, data);
        assert!(result.is_ok());
        assert!(page.is_dirty());

        assert_eq!(&page.data[offset..offset + data.len()], data);
        assert_eq!(page.data[offset - 1], 0);
        assert_eq!(page.data[offset + data.len()], 0);
    }

    #[test]
    fn test_page_write_data_exceeds_size() {
        let mut page = Page::new(1);
        let data = b"test";
        let offset = PAGE_SIZE as usize - 2;

        let result = page.write_data(offset, data);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Data exceeds page size");
        assert!(!page.is_dirty());
    }

    #[test]
    fn test_page_read_data() {
        let mut page = Page::new(1);
        let data = b"hello world";

        page.write_data(0, data).unwrap();

        let read_result = page.read_data(0, data.len());
        assert!(read_result.is_ok());
        assert_eq!(read_result.unwrap(), data);
    }

    #[test]
    fn test_page_read_data_with_offset() {
        let mut page = Page::new(1);
        let data = b"test data";
        let offset = 50;

        page.write_data(offset, data).unwrap();

        let read_result = page.read_data(offset, data.len());
        assert!(read_result.is_ok());
        assert_eq!(read_result.unwrap(), data);
    }

    #[test]
    fn test_page_read_data_exceeds_size() {
        let page = Page::new(1);
        let offset = PAGE_SIZE as usize - 2;
        let length = 5;

        let result = page.read_data(offset, length);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Read exceeds page size");
    }

    #[test]
    fn test_page_read_empty_data() {
        let page = Page::new(1);

        let result = page.read_data(0, 0);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), &[] as &[u8]);
    }

    #[test]
    fn test_page_size() {
        let page = Page::new(1);
        assert_eq!(page.size(), PAGE_SIZE as usize);
    }

    #[test]
    fn test_page_free_space() {
        let page = Page::new(1);
        assert_eq!(page.free_space(), 0);

        let data = vec![1, 2, 3];
        let page2 = Page::from_data(1, data);
        assert_eq!(page2.free_space(), 0);
    }

    #[test]
    fn test_page_multiple_writes() {
        let mut page = Page::new(1);

        page.write_data(0, b"hello").unwrap();
        page.write_data(10, b"world").unwrap();

        assert_eq!(page.read_data(0, 5).unwrap(), b"hello");
        assert_eq!(page.read_data(10, 5).unwrap(), b"world");
        assert_eq!(page.data[5], 0);
        assert_eq!(page.data[9], 0);
    }

    #[test]
    fn test_page_overwrite_data() {
        let mut page = Page::new(1);

        page.write_data(0, b"hello").unwrap();
        page.write_data(0, b"world").unwrap();

        assert_eq!(page.read_data(0, 5).unwrap(), b"world");
    }

    #[test]
    fn test_page_clone() {
        let mut page = Page::new(42);
        page.write_data(0, b"test").unwrap();

        let cloned = page.clone();

        assert_eq!(cloned.id, page.id);
        assert_eq!(cloned.data, page.data);
        assert_eq!(cloned.dirty, page.dirty);
        assert_eq!(cloned.is_dirty(), page.is_dirty());
    }

    #[test]
    fn test_page_debug_format() {
        let page = Page::new(123);
        let debug_str = format!("{page:?}");

        assert!(debug_str.contains("Page"));
        assert!(debug_str.contains("id: 123"));
        assert!(debug_str.contains("dirty: false"));
    }
}

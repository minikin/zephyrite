//! Buffer pool for caching pages in memory
//!
//! The buffer pool manages a cache of frequently accessed pages to reduce
//! disk I/O and improve performance. It uses an LRU eviction policy.

use super::page::Page;
use crate::storage::error::StorageResult;
use std::collections::HashMap;
use tracing::warn;

/// Buffer pool for caching pages in memory
///
/// The buffer pool maintains a cache of pages in memory to reduce disk I/O.
/// It uses a Least Recently Used (LRU) eviction policy when the cache is full.
pub struct BufferPool {
    /// Cached pages indexed by page ID
    pages: HashMap<u64, Page>,
    /// Maximum number of pages to cache
    capacity: usize,
    /// Access order for LRU eviction (most recent at end)
    // TODO: Consider using a VecDeque or a dedicated LRU cache structure to achieve O(1) queue operations.
    access_order: Vec<u64>,
}

impl BufferPool {
    /// Create a new buffer pool with the specified capacity
    #[must_use]
    pub fn new(capacity: usize) -> Self {
        BufferPool {
            pages: HashMap::new(),
            capacity,
            access_order: Vec::with_capacity(capacity),
        }
    }

    /// Get a page from the buffer pool if it exists
    ///
    /// This will update the access order for LRU tracking.
    pub fn get_page(&mut self, page_id: u64) -> Option<&mut Page> {
        match self.pages.get_mut(&page_id) {
            Some(page) => {
                self.access_order.retain(|&id| id != page_id);
                self.access_order.push(page_id);
                Some(page)
            }
            None => None,
        }
    }

    /// Insert a page into the buffer pool
    ///
    /// If the buffer pool is at capacity, this will evict the least recently used page.
    ///
    /// # Errors
    ///
    /// Returns an error if the page cannot be inserted into the buffer pool.
    pub fn insert_page(&mut self, page: Page) -> StorageResult<()> {
        let page_id = page.id;

        // If capacity is 0, don't store anything
        if self.capacity == 0 {
            return Ok(());
        }

        // Evict pages if necessary
        while self.pages.len() >= self.capacity {
            if let Some(&lru_page_id) = self.access_order.first() {
                self.access_order.remove(0);
                if let Some(evicted_page) = self.pages.remove(&lru_page_id) {
                    if evicted_page.is_dirty() {
                        // TODO: write dirty pages back to disk here
                        // Evicting dirty pages without writing back risks data loss.
                        // Consider implementing the disk write here or making the warning behavior explicit in the API documentation.
                        warn!("Evicting dirty page {} - changes may be lost", lru_page_id);
                    }
                }
            } else {
                break;
            }
        }

        self.pages.insert(page_id, page);
        self.access_order.retain(|&id| id != page_id);
        self.access_order.push(page_id);
        Ok(())
    }

    /// Mark a page as dirty in the buffer pool
    pub fn mark_dirty(&mut self, page_id: u64) {
        if let Some(page) = self.pages.get_mut(&page_id) {
            page.mark_dirty();
        }
    }

    /// Get a list of all dirty page IDs
    #[must_use]
    pub fn get_dirty_pages(&self) -> Vec<u64> {
        self.pages
            .iter()
            .filter(|(_, page)| page.is_dirty())
            .map(|(&id, _)| id)
            .collect()
    }

    /// Remove a page from the buffer pool
    pub fn remove_page(&mut self, page_id: u64) -> Option<Page> {
        self.access_order.retain(|&id| id != page_id);
        self.pages.remove(&page_id)
    }

    /// Check if a page is cached
    #[must_use]
    pub fn contains_page(&self, page_id: u64) -> bool {
        self.pages.contains_key(&page_id)
    }

    /// Get the number of pages currently cached
    #[must_use]
    pub fn cached_page_count(&self) -> usize {
        self.pages.len()
    }

    /// Get the capacity of the buffer pool
    #[must_use]
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// Clear all pages from the buffer pool
    ///
    /// This will log warnings for any dirty pages that are cleared.
    pub fn clear(&mut self) {
        for (page_id, page) in &self.pages {
            if page.is_dirty() {
                warn!("Clearing dirty page {} - changes may be lost", page_id);
            }
        }
        self.pages.clear();
        self.access_order.clear();
    }

    /// Get statistics about the buffer pool
    #[must_use]
    pub fn stats(&self) -> BufferPoolStats {
        let dirty_count = self.pages.values().filter(|page| page.is_dirty()).count();

        BufferPoolStats {
            capacity: self.capacity,
            cached_pages: self.pages.len(),
            dirty_pages: dirty_count,
            hit_ratio: 0.0, // TODO: Track hits/misses for accurate hit ratio
        }
    }

    /// Flush all dirty pages
    ///
    /// # Errors
    ///
    /// Returns an error if any dirty pages cannot be flushed to disk.
    pub fn flush_dirty_pages(&mut self) -> StorageResult<Vec<u64>> {
        let dirty_page_ids: Vec<u64> = self.get_dirty_pages();

        // TODO write these pages to disk here
        // For now, just clear the dirty flags
        for page_id in &dirty_page_ids {
            if let Some(page) = self.pages.get_mut(page_id) {
                page.clear_dirty();
            }
        }

        Ok(dirty_page_ids)
    }
}

/// Statistics about the buffer pool
#[derive(Debug, Clone)]
pub struct BufferPoolStats {
    /// Maximum number of pages that can be cached
    pub capacity: usize,
    /// Current number of pages cached
    pub cached_pages: usize,
    /// Number of dirty pages in cache
    pub dirty_pages: usize,
    /// Cache hit ratio (0.0 to 1.0)
    pub hit_ratio: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_page(id: u64) -> Page {
        Page::new(id)
    }

    fn create_dirty_page(id: u64) -> Page {
        let mut page = Page::new(id);
        page.mark_dirty();
        page
    }

    #[test]
    fn test_new_buffer_pool() {
        let pool = BufferPool::new(10);
        assert_eq!(pool.capacity(), 10);
        assert_eq!(pool.cached_page_count(), 0);
        assert!(!pool.contains_page(1));
    }

    #[test]
    fn test_insert_and_get_page() {
        let mut pool = BufferPool::new(3);
        let page = create_test_page(1);

        assert!(pool.insert_page(page).is_ok());
        assert_eq!(pool.cached_page_count(), 1);
        assert!(pool.contains_page(1));

        let retrieved_page = pool.get_page(1);
        assert!(retrieved_page.is_some());
        assert_eq!(retrieved_page.unwrap().id, 1);
    }

    #[test]
    fn test_get_nonexistent_page() {
        let mut pool = BufferPool::new(3);
        assert!(pool.get_page(999).is_none());
    }

    #[test]
    fn test_lru_eviction() {
        let mut pool = BufferPool::new(2);

        // Fill the pool to capacity
        assert!(pool.insert_page(create_test_page(1)).is_ok());
        assert!(pool.insert_page(create_test_page(2)).is_ok());
        assert_eq!(pool.cached_page_count(), 2);

        // Access page 1 to make it more recently used
        assert!(pool.get_page(1).is_some());

        // Insert another page, should evict page 2 (least recently used)
        assert!(pool.insert_page(create_test_page(3)).is_ok());
        assert_eq!(pool.cached_page_count(), 2);
        assert!(pool.contains_page(1));
        assert!(!pool.contains_page(2)); // Should be evicted
        assert!(pool.contains_page(3));
    }

    #[test]
    fn test_mark_dirty() {
        let mut pool = BufferPool::new(3);
        let page = create_test_page(1);

        assert!(pool.insert_page(page).is_ok());

        // Initially not dirty
        assert_eq!(pool.get_dirty_pages().len(), 0);

        // Mark as dirty
        pool.mark_dirty(1);
        let dirty_pages = pool.get_dirty_pages();
        assert_eq!(dirty_pages.len(), 1);
        assert!(dirty_pages.contains(&1));
    }

    #[test]
    fn test_mark_dirty_nonexistent_page() {
        let mut pool = BufferPool::new(3);

        // Marking a non-existent page as dirty should not panic
        pool.mark_dirty(999);
        assert_eq!(pool.get_dirty_pages().len(), 0);
    }

    #[test]
    fn test_get_dirty_pages() {
        let mut pool = BufferPool::new(5);

        assert!(pool.insert_page(create_test_page(1)).is_ok());
        assert!(pool.insert_page(create_dirty_page(2)).is_ok());
        assert!(pool.insert_page(create_test_page(3)).is_ok());
        assert!(pool.insert_page(create_dirty_page(4)).is_ok());

        let dirty_pages = pool.get_dirty_pages();
        assert_eq!(dirty_pages.len(), 2);
        assert!(dirty_pages.contains(&2));
        assert!(dirty_pages.contains(&4));
        assert!(!dirty_pages.contains(&1));
        assert!(!dirty_pages.contains(&3));
    }

    #[test]
    fn test_remove_page() {
        let mut pool = BufferPool::new(3);
        let page = create_test_page(1);

        assert!(pool.insert_page(page).is_ok());
        assert!(pool.contains_page(1));
        assert_eq!(pool.cached_page_count(), 1);

        // Remove the page
        let removed_page = pool.remove_page(1);
        assert!(removed_page.is_some());
        assert_eq!(removed_page.unwrap().id, 1);
        assert!(!pool.contains_page(1));
        assert_eq!(pool.cached_page_count(), 0);
    }

    #[test]
    fn test_remove_nonexistent_page() {
        let mut pool = BufferPool::new(3);
        let removed_page = pool.remove_page(999);
        assert!(removed_page.is_none());
    }

    #[test]
    fn test_clear() {
        let mut pool = BufferPool::new(3);

        assert!(pool.insert_page(create_test_page(1)).is_ok());
        assert!(pool.insert_page(create_dirty_page(2)).is_ok());
        assert_eq!(pool.cached_page_count(), 2);

        pool.clear();
        assert_eq!(pool.cached_page_count(), 0);
        assert!(!pool.contains_page(1));
        assert!(!pool.contains_page(2));
        assert_eq!(pool.get_dirty_pages().len(), 0);
    }

    #[test]
    fn test_stats() {
        let mut pool = BufferPool::new(5);

        assert!(pool.insert_page(create_test_page(1)).is_ok());
        assert!(pool.insert_page(create_dirty_page(2)).is_ok());
        assert!(pool.insert_page(create_test_page(3)).is_ok());

        let stats = pool.stats();
        assert_eq!(stats.capacity, 5);
        assert_eq!(stats.cached_pages, 3);
        assert_eq!(stats.dirty_pages, 1);
        assert!((stats.hit_ratio - 0.0).abs() < f64::EPSILON); // Not implemented yet
    }

    #[test]
    fn test_flush_dirty_pages() {
        let mut pool = BufferPool::new(5);

        assert!(pool.insert_page(create_test_page(1)).is_ok());
        assert!(pool.insert_page(create_dirty_page(2)).is_ok());
        assert!(pool.insert_page(create_dirty_page(3)).is_ok());

        let initial_dirty = pool.get_dirty_pages();
        assert_eq!(initial_dirty.len(), 2);

        let flushed_result = pool.flush_dirty_pages();
        assert!(flushed_result.is_ok());
        let flushed_pages = flushed_result.unwrap();
        assert_eq!(flushed_pages.len(), 2);
        assert!(flushed_pages.contains(&2));
        assert!(flushed_pages.contains(&3));

        assert_eq!(pool.get_dirty_pages().len(), 0);
    }

    #[test]
    fn test_zero_capacity_pool() {
        let mut pool = BufferPool::new(0);
        let page = create_test_page(1);

        // Should not be able to insert into zero-capacity pool
        assert!(pool.insert_page(page).is_ok()); // Function succeeds but page is not stored
        assert_eq!(pool.cached_page_count(), 0);
        assert!(!pool.contains_page(1));
    }

    #[test]
    fn test_lru_order_with_multiple_accesses() {
        let mut pool = BufferPool::new(3);

        assert!(pool.insert_page(create_test_page(1)).is_ok());
        assert!(pool.insert_page(create_test_page(2)).is_ok());
        assert!(pool.insert_page(create_test_page(3)).is_ok());

        // Access pages in specific order to set up LRU state
        assert!(pool.get_page(1).is_some()); // 1 becomes most recent
        assert!(pool.get_page(2).is_some()); // 2 becomes most recent
        // 3 is least recent, then 1, then 2 (most recent)

        // Insert new page - should evict page 3
        assert!(pool.insert_page(create_test_page(4)).is_ok());
        assert!(!pool.contains_page(3));
        assert!(pool.contains_page(1));
        assert!(pool.contains_page(2));
        assert!(pool.contains_page(4));
    }

    #[test]
    fn test_duplicate_page_insertion() {
        let mut pool = BufferPool::new(3);

        // Insert page with ID 1
        assert!(pool.insert_page(create_test_page(1)).is_ok());
        assert_eq!(pool.cached_page_count(), 1);

        // Insert another page with the same ID - should replace the existing one
        let mut new_page = create_test_page(1);
        new_page.mark_dirty();
        assert!(pool.insert_page(new_page).is_ok());
        assert_eq!(pool.cached_page_count(), 1);

        // The page should now be dirty
        let dirty_pages = pool.get_dirty_pages();
        assert_eq!(dirty_pages.len(), 1);
        assert!(dirty_pages.contains(&1));
    }

    #[test]
    fn test_access_order_updates() {
        let mut pool = BufferPool::new(3);

        assert!(pool.insert_page(create_test_page(1)).is_ok());
        assert!(pool.insert_page(create_test_page(2)).is_ok());

        // Access page 1 multiple times
        assert!(pool.get_page(1).is_some());
        assert!(pool.get_page(1).is_some());
        assert!(pool.get_page(1).is_some());

        assert!(pool.insert_page(create_test_page(3)).is_ok());

        // Insert fourth page - should evict page 2 (least recently used)
        assert!(pool.insert_page(create_test_page(4)).is_ok());
        assert!(pool.contains_page(1)); // Recently accessed
        assert!(!pool.contains_page(2)); // Should be evicted
        assert!(pool.contains_page(3));
        assert!(pool.contains_page(4));
    }
}

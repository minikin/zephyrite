/// Page management utilities
#[derive(Debug, Default)]
pub struct PageManager {
    /// Next page ID to allocate
    next_page_id: u64,
    /// List of free page IDs
    free_pages: Vec<u64>,
}

impl PageManager {
    /// Creates a new page manager
    #[must_use]
    pub fn new() -> Self {
        Self {
            // Page 0 is reserved for the header
            next_page_id: 1,
            free_pages: Vec::new(),
        }
    }

    /// Create a new page manager with existing state
    #[must_use]
    pub fn with_state(next_page_id: u64, free_pages: Vec<u64>) -> Self {
        Self {
            next_page_id,
            free_pages,
        }
    }

    /// Allocates a new page ID
    #[must_use]
    pub fn allocate_page(&mut self) -> u64 {
        if let Some(id) = self.free_pages.pop() {
            return id;
        }

        let page_id = self.next_page_id;
        self.next_page_id += 1;
        page_id
    }

    /// Frees a page ID
    pub fn free_page(&mut self, id: u64) {
        if !self.free_pages.contains(&id) {
            self.free_pages.push(id);
            self.free_pages.sort_unstable(); // Keep free pages sorted
        }
    }
    /// Get the next page ID to allocate
    #[must_use]
    pub fn next_page_id(&self) -> u64 {
        self.next_page_id
    }

    /// Get the list of free pages
    #[must_use]
    pub fn free_pages(&self) -> &[u64] {
        &self.free_pages
    }

    /// Get the total number of pages managed
    #[must_use]
    pub fn total_pages(&self) -> u64 {
        self.next_page_id - 1 + self.free_pages.len() as u64
    }

    /// Get the number of free pages
    #[must_use]
    pub fn free_page_count(&self) -> usize {
        self.free_pages.len()
    }
}

#[cfg(test)]
mod page_manager_tests {
    use super::*;

    #[test]
    fn test_page_manager_new() {
        let manager = PageManager::new();

        assert_eq!(manager.next_page_id(), 1);
        assert_eq!(manager.free_page_count(), 0);
        assert_eq!(manager.free_pages().len(), 0);
        assert_eq!(manager.total_pages(), 0);
    }

    #[test]
    fn test_page_manager_with_state() {
        let free_pages = vec![5, 10, 15];
        let manager = PageManager::with_state(20, free_pages.clone());

        assert_eq!(manager.next_page_id(), 20);
        assert_eq!(manager.free_page_count(), 3);
        assert_eq!(manager.free_pages(), &[5, 10, 15]);
        assert_eq!(manager.total_pages(), 22);
    }

    #[test]
    fn test_page_manager_allocate_page() {
        let mut manager = PageManager::new();

        let page_id1 = manager.allocate_page();
        assert_eq!(page_id1, 1);
        assert_eq!(manager.next_page_id(), 2);

        let page_id2 = manager.allocate_page();
        assert_eq!(page_id2, 2);
        assert_eq!(manager.next_page_id(), 3);

        assert_eq!(manager.total_pages(), 2);
    }

    #[test]
    fn test_page_manager_free_page() {
        let mut manager = PageManager::new();

        let _page1 = manager.allocate_page();
        let _page2 = manager.allocate_page();
        let _page3 = manager.allocate_page();

        assert_eq!(manager.free_page_count(), 0);

        manager.free_page(2);
        assert_eq!(manager.free_page_count(), 1);
        assert_eq!(manager.free_pages(), &[2]);

        manager.free_page(1);
        assert_eq!(manager.free_page_count(), 2);
        assert_eq!(manager.free_pages(), &[1, 2]);
    }

    #[test]
    fn test_page_manager_allocate_from_free_list() {
        let mut manager = PageManager::new();

        let _page1 = manager.allocate_page();
        let _page2 = manager.allocate_page();
        let _page3 = manager.allocate_page();

        manager.free_page(2);
        manager.free_page(1);

        assert_eq!(manager.free_page_count(), 2);

        let reused_page = manager.allocate_page();
        assert_eq!(reused_page, 2);
        assert_eq!(manager.free_page_count(), 1);
        assert_eq!(manager.free_pages(), &[1]);

        let reused_page2 = manager.allocate_page();
        assert_eq!(reused_page2, 1);
        assert_eq!(manager.free_page_count(), 0);
        assert_eq!(manager.free_pages().len(), 0);

        let new_page = manager.allocate_page();
        assert_eq!(new_page, 4);
        assert_eq!(manager.next_page_id(), 5);
    }

    #[test]
    fn test_page_manager_free_page_no_duplicates() {
        let mut manager = PageManager::new();

        let _page = manager.allocate_page();
        manager.free_page(1);
        manager.free_page(1);

        assert_eq!(manager.free_page_count(), 1);
        assert_eq!(manager.free_pages(), &[1]);
    }

    #[test]
    fn test_page_manager_free_pages_sorted() {
        let mut manager = PageManager::new();

        manager.free_page(10);
        manager.free_page(5);
        manager.free_page(15);
        manager.free_page(1);

        assert_eq!(manager.free_pages(), &[1, 5, 10, 15]);
    }

    #[test]
    fn test_page_manager_total_pages() {
        let mut manager = PageManager::new();

        // Initially no pages allocated
        assert_eq!(manager.total_pages(), 0);

        // Allocate first page (ID 1)
        let _page1 = manager.allocate_page();
        assert_eq!(manager.total_pages(), 1); // next_page_id=2, free_pages=0, total = 2-1+0 = 1

        // Allocate second page (ID 2)
        let _page2 = manager.allocate_page();
        assert_eq!(manager.total_pages(), 2); // next_page_id=3, free_pages=0, total = 3-1+0 = 2

        // Free page 1
        manager.free_page(1);
        assert_eq!(manager.total_pages(), 3); // next_page_id=3, free_pages=1, total = 3-1+1 = 3

        // Allocate again (should reuse page 1)
        let _reused_page = manager.allocate_page();
        assert_eq!(manager.total_pages(), 2); // next_page_id=3, free_pages=0, total = 3-1+0 = 2
    }

    #[test]
    fn test_page_manager_complex_allocation_pattern() {
        let mut manager = PageManager::new();

        let page1 = manager.allocate_page();
        let page2 = manager.allocate_page();
        let page3 = manager.allocate_page();
        let page4 = manager.allocate_page();

        assert_eq!(page1, 1);
        assert_eq!(page2, 2);
        assert_eq!(page3, 3);
        assert_eq!(page4, 4);

        manager.free_page(page2);
        manager.free_page(page4);

        assert_eq!(manager.free_page_count(), 2);
        assert_eq!(manager.free_pages(), &[2, 4]);

        let reused1 = manager.allocate_page();
        let reused2 = manager.allocate_page();

        assert_eq!(reused1, 4);
        assert_eq!(reused2, 2);
        assert_eq!(manager.free_page_count(), 0);

        let new_page = manager.allocate_page();
        assert_eq!(new_page, 5);
        assert_eq!(manager.next_page_id(), 6);
    }

    #[test]
    fn test_page_manager_page_zero_reserved() {
        let manager = PageManager::new();

        assert_eq!(manager.next_page_id(), 1);

        let mut manager = manager;
        let first_page = manager.allocate_page();
        assert_eq!(first_page, 1);
        assert_ne!(first_page, 0);
    }
}

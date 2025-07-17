//! Index management for disk storage
//!
//! The index provides fast key lookups by mapping keys to their location
//! on disk (page ID and offset within the page).

use std::collections::HashMap;

/// Index entry pointing to a data page
///
/// Each index entry contains the information needed to locate
/// a value stored on disk.
#[derive(Debug, Clone)]
pub struct IndexEntry {
    /// The key this entry refers to
    pub key: String,
    /// Page ID where the data is stored
    pub page_id: u64,
    /// Offset within the page where the data starts
    pub offset: u16,
    /// Size of the stored data in bytes
    pub size: u16,
}

impl IndexEntry {
    /// Create a new index entry
    #[must_use]
    pub fn new(key: String, page_id: u64, offset: u16, size: u16) -> Self {
        Self {
            key,
            page_id,
            offset,
            size,
        }
    }

    /// Get the end offset of the data (offset + size)
    #[must_use]
    pub fn end_offset(&self) -> u16 {
        self.offset.saturating_add(self.size)
    }

    /// Check if this entry overlaps with another entry on the same page
    #[must_use]
    pub fn overlaps_with(&self, other: &IndexEntry) -> bool {
        if self.page_id != other.page_id {
            return false;
        }

        let self_end = self.end_offset();
        let other_end = other.end_offset();

        !(self_end <= other.offset || other_end <= self.offset)
    }
}

/// In-memory index for fast key lookups
///
/// This index maintains a mapping from keys to their storage locations.
// TODO: Consider persisted to disk as well
#[derive(Debug, Default)]
pub struct Index {
    /// Map from key to index entry
    entries: HashMap<String, IndexEntry>,
}

impl Index {
    /// Create a new empty index
    #[must_use]
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
        }
    }

    /// Create an index with initial capacity
    #[must_use]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            entries: HashMap::with_capacity(capacity),
        }
    }

    /// Insert or update an index entry
    pub fn insert(&mut self, key: String, entry: IndexEntry) -> Option<IndexEntry> {
        self.entries.insert(key, entry)
    }

    /// Get an index entry by key
    #[must_use]
    pub fn get(&self, key: &str) -> Option<&IndexEntry> {
        self.entries.get(key)
    }

    /// Remove an index entry by key
    #[must_use]
    pub fn remove(&mut self, key: &str) -> Option<IndexEntry> {
        self.entries.remove(key)
    }

    /// Check if a key exists in the index
    #[must_use]
    pub fn contains_key(&self, key: &str) -> bool {
        self.entries.contains_key(key)
    }

    /// Get all keys in the index
    pub fn keys(&self) -> impl Iterator<Item = &String> {
        self.entries.keys()
    }

    /// Get all index entries
    #[must_use]
    pub fn entries(&self) -> &HashMap<String, IndexEntry> {
        &self.entries
    }

    /// Get the number of entries in the index
    #[must_use]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Check if the index is empty
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Clear all entries from the index
    pub fn clear(&mut self) {
        self.entries.clear();
    }

    /// Get entries that are stored on a specific page
    #[must_use]
    pub fn entries_on_page(&self, page_id: u64) -> Vec<&IndexEntry> {
        self.entries
            .values()
            .filter(|entry| entry.page_id == page_id)
            .collect()
    }

    /// Find pages that contain data (for iteration or compaction)
    #[must_use]
    pub fn used_pages(&self) -> Vec<u64> {
        let mut pages: Vec<u64> = self.entries.values().map(|entry| entry.page_id).collect();
        pages.sort_unstable();
        pages.dedup();
        pages
    }

    /// Get statistics about the index
    #[must_use]
    pub fn stats(&self) -> IndexStats {
        if self.entries.is_empty() {
            return IndexStats {
                entry_count: 0,
                page_count: 0,
                total_data_size: 0,
                average_key_length: 0.0,
                average_value_size: 0.0,
                max_value_size: 0,
                min_value_size: 0,
                average_entries_per_page: 0.0,
            };
        }

        // Use a HashSet to count unique pages efficiently
        let mut unique_pages = std::collections::HashSet::new();
        let mut total_key_length = 0;
        let mut total_data_size = 0;
        let mut min_value_size = u16::MAX;
        let mut max_value_size = 0;

        for entry in self.entries.values() {
            unique_pages.insert(entry.page_id);
            total_key_length += entry.key.len();
            total_data_size += entry.size as usize;
            min_value_size = min_value_size.min(entry.size);
            max_value_size = max_value_size.max(entry.size);
        }

        let entry_count = self.entries.len();
        let page_count = unique_pages.len();

        #[allow(clippy::cast_precision_loss)]
        IndexStats {
            entry_count,
            page_count,
            total_data_size,
            average_key_length: total_key_length as f64 / entry_count as f64,
            average_value_size: total_data_size as f64 / entry_count as f64,
            max_value_size,
            min_value_size,
            average_entries_per_page: if page_count > 0 {
                entry_count as f64 / page_count as f64
            } else {
                0.0
            },
        }
    }

    /// Validate the index for consistency
    #[must_use]
    pub fn validate(&self) -> Vec<String> {
        let mut errors = Vec::new();

        // Check for overlapping entries on the same page
        let mut page_entries: HashMap<u64, Vec<&IndexEntry>> = HashMap::new();
        for entry in self.entries.values() {
            page_entries.entry(entry.page_id).or_default().push(entry);
        }

        for (page_id, entries) in page_entries {
            for i in 0..entries.len() {
                for j in i + 1..entries.len() {
                    if entries[i].overlaps_with(entries[j]) {
                        errors.push(format!(
                            "Overlapping entries on page {}: {} and {}",
                            page_id, entries[i].key, entries[j].key
                        ));
                    }
                }
            }
        }

        errors
    }
}

/// Statistics about the index
#[derive(Debug, Clone)]
pub struct IndexStats {
    /// Number of entries in the index
    pub entry_count: usize,
    /// Number of pages that contain data
    pub page_count: usize,
    /// Total size of all stored data in bytes
    pub total_data_size: usize,
    /// Average length of keys in characters
    pub average_key_length: f64,
    /// Average size of stored values in bytes
    pub average_value_size: f64,
    /// Maximum value size in bytes
    pub max_value_size: u16,
    /// Minimum value size in bytes
    pub min_value_size: u16,
    /// Average number of entries per page
    pub average_entries_per_page: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_index_entry_new() {
        let entry = IndexEntry::new("test_key".to_string(), 1, 100, 50);
        assert_eq!(entry.key, "test_key");
        assert_eq!(entry.page_id, 1);
        assert_eq!(entry.offset, 100);
        assert_eq!(entry.size, 50);
    }

    #[test]
    fn test_index_entry_end_offset() {
        let entry = IndexEntry::new("test".to_string(), 1, 100, 50);
        assert_eq!(entry.end_offset(), 150);
    }

    #[test]
    fn test_index_entry_end_offset_overflow() {
        let entry = IndexEntry::new("test".to_string(), 1, u16::MAX - 10, 20);
        assert_eq!(entry.end_offset(), u16::MAX);
    }

    #[test]
    fn test_index_entry_overlaps_with_different_pages() {
        let entry1 = IndexEntry::new("key1".to_string(), 1, 100, 50);
        let entry2 = IndexEntry::new("key2".to_string(), 2, 100, 50);
        assert!(!entry1.overlaps_with(&entry2));
    }

    #[test]
    fn test_index_entry_overlaps_with_same_page_no_overlap() {
        let entry1 = IndexEntry::new("key1".to_string(), 1, 100, 50);
        let entry2 = IndexEntry::new("key2".to_string(), 1, 150, 50);
        assert!(!entry1.overlaps_with(&entry2));
    }

    #[test]
    fn test_index_entry_overlaps_with_same_page_overlap() {
        let entry1 = IndexEntry::new("key1".to_string(), 1, 100, 50);
        let entry2 = IndexEntry::new("key2".to_string(), 1, 120, 50);
        assert!(entry1.overlaps_with(&entry2));
    }

    #[test]
    fn test_index_entry_overlaps_with_adjacent_entries() {
        let entry1 = IndexEntry::new("key1".to_string(), 1, 100, 50);
        let entry2 = IndexEntry::new("key2".to_string(), 1, 150, 50);
        assert!(!entry1.overlaps_with(&entry2));
    }

    #[test]
    fn test_index_new() {
        let index = Index::new();
        assert!(index.is_empty());
        assert_eq!(index.len(), 0);
    }

    #[test]
    fn test_index_with_capacity() {
        let index = Index::with_capacity(10);
        assert!(index.is_empty());
        assert_eq!(index.len(), 0);
    }

    #[test]
    fn test_index_insert_and_get() {
        let mut index = Index::new();
        let entry = IndexEntry::new("test_key".to_string(), 1, 100, 50);

        assert!(
            index
                .insert("test_key".to_string(), entry.clone())
                .is_none()
        );
        assert_eq!(index.len(), 1);
        assert!(!index.is_empty());

        let retrieved = index.get("test_key").unwrap();
        assert_eq!(retrieved.key, "test_key");
        assert_eq!(retrieved.page_id, 1);
        assert_eq!(retrieved.offset, 100);
        assert_eq!(retrieved.size, 50);
    }

    #[test]
    fn test_index_insert_duplicate_key() {
        let mut index = Index::new();
        let entry1 = IndexEntry::new("test_key".to_string(), 1, 100, 50);
        let entry2 = IndexEntry::new("test_key".to_string(), 2, 200, 75);

        assert!(
            index
                .insert("test_key".to_string(), entry1.clone())
                .is_none()
        );
        let old_entry = index
            .insert("test_key".to_string(), entry2.clone())
            .unwrap();

        assert_eq!(old_entry.page_id, 1);
        assert_eq!(index.get("test_key").unwrap().page_id, 2);
    }

    #[test]
    fn test_index_get_nonexistent() {
        let index = Index::new();
        assert!(index.get("nonexistent").is_none());
    }

    #[test]
    fn test_index_contains_key() {
        let mut index = Index::new();
        let entry = IndexEntry::new("test_key".to_string(), 1, 100, 50);

        assert!(!index.contains_key("test_key"));
        index.insert("test_key".to_string(), entry);
        assert!(index.contains_key("test_key"));
    }

    #[test]
    fn test_index_remove() {
        let mut index = Index::new();
        let entry = IndexEntry::new("test_key".to_string(), 1, 100, 50);

        index.insert("test_key".to_string(), entry.clone());
        assert!(index.contains_key("test_key"));

        let removed = index.remove("test_key").unwrap();
        assert_eq!(removed.key, "test_key");
        assert!(!index.contains_key("test_key"));
        assert!(index.is_empty());
    }

    #[test]
    fn test_index_remove_nonexistent() {
        let mut index = Index::new();
        assert!(index.remove("nonexistent").is_none());
    }

    #[test]
    fn test_index_keys() {
        let mut index = Index::new();
        let entry1 = IndexEntry::new("key1".to_string(), 1, 100, 50);
        let entry2 = IndexEntry::new("key2".to_string(), 2, 200, 75);

        index.insert("key1".to_string(), entry1);
        index.insert("key2".to_string(), entry2);

        let keys: Vec<_> = index.keys().collect();
        assert_eq!(keys.len(), 2);
        assert!(keys.contains(&&"key1".to_string()));
        assert!(keys.contains(&&"key2".to_string()));
    }

    #[test]
    fn test_index_entries() {
        let mut index = Index::new();
        let entry = IndexEntry::new("test_key".to_string(), 1, 100, 50);

        index.insert("test_key".to_string(), entry.clone());

        let entries = index.entries();
        assert_eq!(entries.len(), 1);
        assert!(entries.contains_key("test_key"));
    }

    #[test]
    fn test_index_clear() {
        let mut index = Index::new();
        let entry = IndexEntry::new("test_key".to_string(), 1, 100, 50);

        index.insert("test_key".to_string(), entry);
        assert!(!index.is_empty());

        index.clear();
        assert!(index.is_empty());
        assert_eq!(index.len(), 0);
    }

    #[test]
    fn test_index_entries_on_page() {
        let mut index = Index::new();
        let entry1 = IndexEntry::new("key1".to_string(), 1, 100, 50);
        let entry2 = IndexEntry::new("key2".to_string(), 1, 200, 75);
        let entry3 = IndexEntry::new("key3".to_string(), 2, 100, 50);

        index.insert("key1".to_string(), entry1);
        index.insert("key2".to_string(), entry2);
        index.insert("key3".to_string(), entry3);

        let page1_entries = index.entries_on_page(1);
        assert_eq!(page1_entries.len(), 2);

        let page2_entries = index.entries_on_page(2);
        assert_eq!(page2_entries.len(), 1);

        let page3_entries = index.entries_on_page(3);
        assert_eq!(page3_entries.len(), 0);
    }

    #[test]
    fn test_index_used_pages() {
        let mut index = Index::new();
        let entry1 = IndexEntry::new("key1".to_string(), 3, 100, 50);
        let entry2 = IndexEntry::new("key2".to_string(), 1, 200, 75);
        let entry3 = IndexEntry::new("key3".to_string(), 3, 300, 25);
        let entry4 = IndexEntry::new("key4".to_string(), 2, 100, 50);

        index.insert("key1".to_string(), entry1);
        index.insert("key2".to_string(), entry2);
        index.insert("key3".to_string(), entry3);
        index.insert("key4".to_string(), entry4);

        let used_pages = index.used_pages();
        assert_eq!(used_pages, vec![1, 2, 3]);
    }

    #[test]
    fn test_index_stats_empty() {
        let index = Index::new();
        let stats = index.stats();

        assert_eq!(stats.entry_count, 0);
        assert_eq!(stats.page_count, 0);
        assert_eq!(stats.total_data_size, 0);
        assert!((stats.average_key_length - 0.0).abs() < f64::EPSILON);
        assert!((stats.average_value_size - 0.0).abs() < f64::EPSILON);
        assert_eq!(stats.max_value_size, 0);
        assert_eq!(stats.min_value_size, 0);
        assert!((stats.average_entries_per_page - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_index_stats_single_entry() {
        let mut index = Index::new();
        let entry = IndexEntry::new("test".to_string(), 1, 100, 50);
        index.insert("test".to_string(), entry);

        let stats = index.stats();
        assert_eq!(stats.entry_count, 1);
        assert_eq!(stats.page_count, 1);
        assert_eq!(stats.total_data_size, 50);
        assert!((stats.average_key_length - 4.0).abs() < f64::EPSILON);
        assert!((stats.average_value_size - 50.0).abs() < f64::EPSILON);
        assert_eq!(stats.max_value_size, 50);
        assert_eq!(stats.min_value_size, 50);
        assert!((stats.average_entries_per_page - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_index_stats_multiple_entries() {
        let mut index = Index::new();
        let entry1 = IndexEntry::new("key1".to_string(), 1, 100, 50);
        let entry2 = IndexEntry::new("key22".to_string(), 1, 200, 75);
        let entry3 = IndexEntry::new("key333".to_string(), 2, 100, 25);

        index.insert("key1".to_string(), entry1);
        index.insert("key22".to_string(), entry2);
        index.insert("key333".to_string(), entry3);

        let stats = index.stats();
        assert_eq!(stats.entry_count, 3);
        assert_eq!(stats.page_count, 2);
        assert_eq!(stats.total_data_size, 150);
        assert!((stats.average_key_length - 5.0).abs() < f64::EPSILON);
        assert!((stats.average_value_size - 50.0).abs() < f64::EPSILON);
        assert_eq!(stats.max_value_size, 75);
        assert_eq!(stats.min_value_size, 25);
        assert!((stats.average_entries_per_page - 1.5).abs() < f64::EPSILON);
    }

    #[test]
    fn test_index_validate_no_errors() {
        let mut index = Index::new();
        let entry1 = IndexEntry::new("key1".to_string(), 1, 100, 50);
        let entry2 = IndexEntry::new("key2".to_string(), 1, 200, 50);
        let entry3 = IndexEntry::new("key3".to_string(), 2, 100, 50);

        index.insert("key1".to_string(), entry1);
        index.insert("key2".to_string(), entry2);
        index.insert("key3".to_string(), entry3);

        let errors = index.validate();
        assert!(errors.is_empty());
    }

    #[test]
    fn test_index_validate_overlapping_entries() {
        let mut index = Index::new();
        let entry1 = IndexEntry::new("key1".to_string(), 1, 100, 50);
        let entry2 = IndexEntry::new("key2".to_string(), 1, 120, 50);

        index.insert("key1".to_string(), entry1);
        index.insert("key2".to_string(), entry2);

        let errors = index.validate();
        assert_eq!(errors.len(), 1);
        assert!(errors[0].contains("Overlapping entries on page 1"));
        assert!(errors[0].contains("key1"));
        assert!(errors[0].contains("key2"));
    }

    #[test]
    fn test_index_validate_multiple_overlapping_entries() {
        let mut index = Index::new();
        let entry1 = IndexEntry::new("key1".to_string(), 1, 100, 50);
        let entry2 = IndexEntry::new("key2".to_string(), 1, 120, 50);
        let entry3 = IndexEntry::new("key3".to_string(), 1, 140, 50);

        index.insert("key1".to_string(), entry1);
        index.insert("key2".to_string(), entry2);
        index.insert("key3".to_string(), entry3);

        let errors = index.validate();
        assert!(errors.len() >= 2);
    }

    #[test]
    fn test_index_validate_empty_index() {
        let index = Index::new();
        let errors = index.validate();
        assert!(errors.is_empty());
    }

    #[test]
    fn test_index_entry_clone() {
        let entry1 = IndexEntry::new("test".to_string(), 1, 100, 50);
        let entry2 = entry1.clone();

        assert_eq!(entry1.key, entry2.key);
        assert_eq!(entry1.page_id, entry2.page_id);
        assert_eq!(entry1.offset, entry2.offset);
        assert_eq!(entry1.size, entry2.size);
    }

    #[test]
    fn test_index_stats_clone() {
        let mut index = Index::new();
        let entry = IndexEntry::new("test".to_string(), 1, 100, 50);
        index.insert("test".to_string(), entry);

        let stats1 = index.stats();
        let stats2 = stats1.clone();

        assert_eq!(stats1.entry_count, stats2.entry_count);
        assert_eq!(stats1.page_count, stats2.page_count);
        assert_eq!(stats1.total_data_size, stats2.total_data_size);
    }

    #[test]
    fn test_index_default() {
        let index = Index::default();
        assert!(index.is_empty());
        assert_eq!(index.len(), 0);
    }
}

//! Local index for deduplication and idempotence tracking.
//!
//! This module provides persistent storage of file hashes to enable idempotent
//! operations on network storage. The index maps file hashes to their metadata
//! and is serialized using Bincode for compact binary storage.
//!
//! # Examples
//!
//! Create and use an index:
//! ```no_run
//! # use sift::index::Index;
//! let mut index = Index::new();
//! index.add_entry("abc123".to_string(), "/path/to/file".to_string());
//!
//! if index.contains_hash("abc123") {
//!     println!("File already processed");
//! }
//!
//! index.save_to_file("index.bin")?;
//! # Ok::<(), std::io::Error>(())
//! ```

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::Path;

/// Represents a single entry in the deduplication index.
///
/// # Fields
///
/// * `hash` - The Blake3 hash of the file contents
/// * `file_path` - The path where the file was originally located
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexEntry {
    pub hash: String,
    pub file_path: String,
}

/// A persistent index for tracking processed files and enabling idempotent operations.
///
/// The index stores file hashes and metadata, allowing the application to detect
/// duplicate files and avoid reprocessing them. The index can be saved to and loaded
/// from disk using Bincode serialization.
///
/// # Thread Safety
///
/// This struct is not thread-safe. For concurrent access, wrap it in `Arc<Mutex<>>`.
#[derive(Debug, Serialize, Deserialize)]
pub struct Index {
    /// Map from hash to file information
    entries: HashMap<String, IndexEntry>,
}

impl Index {
    /// Creates a new empty index.
    ///
    /// # Examples
    ///
    /// ```
    /// # use sift::index::Index;
    /// let index = Index::new();
    /// assert!(index.is_empty());
    /// ```
    pub fn new() -> Self {
        Index {
            entries: HashMap::new(),
        }
    }

    /// Checks if a hash already exists in the index.
    ///
    /// # Arguments
    ///
    /// * `hash` - The hash string to check
    ///
    /// # Returns
    ///
    /// `true` if the hash is in the index, `false` otherwise
    ///
    /// # Examples
    ///
    /// ```
    /// # use sift::index::Index;
    /// let mut index = Index::new();
    /// assert!(!index.contains_hash("abc123"));
    /// index.add_entry("abc123".to_string(), "/path".to_string());
    /// assert!(index.contains_hash("abc123"));
    /// ```
    pub fn contains_hash(&self, hash: &str) -> bool {
        self.entries.contains_key(hash)
    }

    /// Adds an entry to the index.
    ///
    /// If an entry with the same hash already exists, it will be overwritten.
    ///
    /// # Arguments
    ///
    /// * `hash` - The Blake3 hash of the file
    /// * `file_path` - The path to the file
    pub fn add_entry(&mut self, hash: String, file_path: String) {
        self.entries.insert(
            hash.clone(),
            IndexEntry {
                hash,
                file_path,
            },
        );
    }

    /// Retrieves an entry from the index by hash.
    ///
    /// # Arguments
    ///
    /// * `hash` - The hash to look up
    ///
    /// # Returns
    ///
    /// * `Some(&IndexEntry)` if the hash exists
    /// * `None` if the hash is not in the index
    pub fn get_entry(&self, hash: &str) -> Option<&IndexEntry> {
        self.entries.get(hash)
    }

    /// Returns the number of entries in the index.
    ///
    /// # Examples
    ///
    /// ```
    /// # use sift::index::Index;
    /// let mut index = Index::new();
    /// assert_eq!(index.len(), 0);
    /// index.add_entry("hash1".to_string(), "/path1".to_string());
    /// assert_eq!(index.len(), 1);
    /// ```
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Returns `true` if the index contains no entries.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Returns an iterator over all entries in the index.
    ///
    /// # Examples
    ///
    /// ```
    /// # use sift::index::Index;
    /// let mut index = Index::new();
    /// index.add_entry("hash1".to_string(), "/path1".to_string());
    /// for entry in index.entries() {
    ///     println!("{}: {}", entry.hash, entry.file_path);
    /// }
    /// ```
    pub fn entries(&self) -> impl Iterator<Item = &IndexEntry> {
        self.entries.values()
    }

    /// Loads an index from a binary file (Bincode format).
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the index file
    ///
    /// # Returns
    ///
    /// * `Ok(Index)` - The loaded index
    /// * `Err(io::Error)` - If the file cannot be read or deserialized
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use sift::index::Index;
    /// let index = Index::load_from_file("index.bin")?;
    /// println!("Loaded {} entries", index.len());
    /// # Ok::<(), std::io::Error>(())
    /// ```
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let data = fs::read(path)?;
        bincode::deserialize(&data)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    }

    /// Saves the index to a binary file (Bincode format).
    ///
    /// # Arguments
    ///
    /// * `path` - Path where the index should be saved
    ///
    /// # Returns
    ///
    /// * `Ok(())` - If the index was successfully saved
    /// * `Err(io::Error)` - If the file cannot be written or serialization fails
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use sift::index::Index;
    /// let mut index = Index::new();
    /// index.add_entry("hash1".to_string(), "/path1".to_string());
    /// index.save_to_file("index.bin")?;
    /// # Ok::<(), std::io::Error>(())
    /// ```
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> io::Result<()> {
        let data = bincode::serialize(self)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        fs::write(path, data)?;
        Ok(())
    }
}

impl Default for Index {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_index_creation() {
        let index = Index::new();
        assert!(index.is_empty());
        assert_eq!(index.len(), 0);
    }

    #[test]
    fn test_add_single_entry() {
        let mut index = Index::new();
        let hash = "abc123".to_string();
        let path = "/photos/img1.jpg".to_string();

        index.add_entry(hash.clone(), path.clone());

        assert!(index.contains_hash(&hash));
        assert_eq!(index.len(), 1);
        assert_eq!(index.get_entry(&hash).unwrap().file_path, path);
    }

    #[test]
    fn test_add_multiple_entries() {
        let mut index = Index::new();
        index.add_entry("hash1".to_string(), "/file1".to_string());
        index.add_entry("hash2".to_string(), "/file2".to_string());
        index.add_entry("hash3".to_string(), "/file3".to_string());

        assert_eq!(index.len(), 3);
        assert!(index.contains_hash("hash1"));
        assert!(index.contains_hash("hash2"));
        assert!(index.contains_hash("hash3"));
    }

    #[test]
    fn test_overwrite_entry() {
        let mut index = Index::new();
        index.add_entry("hash1".to_string(), "/old/path".to_string());
        index.add_entry("hash1".to_string(), "/new/path".to_string());

        assert_eq!(index.len(), 1);
        assert_eq!(index.get_entry("hash1").unwrap().file_path, "/new/path");
    }

    #[test]
    fn test_contains_hash_nonexistent() {
        let index = Index::new();
        assert!(!index.contains_hash("nonexistent"));
    }

    #[test]
    fn test_get_entry_nonexistent() {
        let index = Index::new();
        assert!(index.get_entry("nonexistent").is_none());
    }

    #[test]
    fn test_entries_iterator() {
        let mut index = Index::new();
        index.add_entry("hash1".to_string(), "/file1".to_string());
        index.add_entry("hash2".to_string(), "/file2".to_string());

        let entries: Vec<_> = index.entries().collect();
        assert_eq!(entries.len(), 2);
    }

    #[test]
    fn test_entries_iterator_empty() {
        let index = Index::new();
        let entries: Vec<_> = index.entries().collect();
        assert!(entries.is_empty());
    }

    #[test]
    fn test_persistence_basic() -> io::Result<()> {
        let dir = tempdir()?;
        let index_path = dir.path().join("test.index");

        let mut index = Index::new();
        index.add_entry("hash1".to_string(), "/path/to/file1".to_string());
        index.add_entry("hash2".to_string(), "/path/to/file2".to_string());

        index.save_to_file(&index_path)?;

        let loaded_index = Index::load_from_file(&index_path)?;
        assert_eq!(loaded_index.len(), 2);
        assert!(loaded_index.contains_hash("hash1"));
        assert!(loaded_index.contains_hash("hash2"));

        Ok(())
    }

    #[test]
    fn test_persistence_preserves_data() -> io::Result<()> {
        let dir = tempdir()?;
        let index_path = dir.path().join("test.index");

        let mut index = Index::new();
        index.add_entry("abc123def".to_string(), "/very/long/path/to/file.jpg".to_string());

        index.save_to_file(&index_path)?;

        let loaded = Index::load_from_file(&index_path)?;
        let entry = loaded.get_entry("abc123def").unwrap();
        assert_eq!(entry.file_path, "/very/long/path/to/file.jpg");

        Ok(())
    }

    #[test]
    fn test_persistence_large_index() -> io::Result<()> {
        let dir = tempdir()?;
        let index_path = dir.path().join("large.index");

        let mut index = Index::new();
        for i in 0..1000 {
            index.add_entry(
                format!("hash_{}", i),
                format!("/path/to/file_{}.jpg", i),
            );
        }

        index.save_to_file(&index_path)?;

        let loaded = Index::load_from_file(&index_path)?;
        assert_eq!(loaded.len(), 1000);
        assert!(loaded.contains_hash("hash_500"));
        assert_eq!(loaded.get_entry("hash_999").unwrap().file_path, "/path/to/file_999.jpg");

        Ok(())
    }

    #[test]
    fn test_load_nonexistent_file() {
        let result = Index::load_from_file("/nonexistent/path/index.bin");
        assert!(result.is_err());
    }

    #[test]
    fn test_save_to_nonexistent_directory() {
        let index = Index::new();
        let result = index.save_to_file("/nonexistent/directory/index.bin");
        assert!(result.is_err());
    }
}

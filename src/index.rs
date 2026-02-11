use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::Path;

/// Index entry mapping file hash to metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexEntry {
    pub hash: String,
    pub file_path: String,
}

/// Local index for deduplication and idempotence
#[derive(Debug, Serialize, Deserialize)]
pub struct Index {
    /// Map from hash to file information
    entries: HashMap<String, IndexEntry>,
}

impl Index {
    /// Create a new empty index
    pub fn new() -> Self {
        Index {
            entries: HashMap::new(),
        }
    }

    /// Check if a hash already exists in the index
    pub fn contains_hash(&self, hash: &str) -> bool {
        self.entries.contains_key(hash)
    }

    /// Add a hash entry to the index
    pub fn add_entry(&mut self, hash: String, file_path: String) {
        self.entries.insert(
            hash.clone(),
            IndexEntry {
                hash,
                file_path,
            },
        );
    }

    /// Get an entry by hash
    pub fn get_entry(&self, hash: &str) -> Option<&IndexEntry> {
        self.entries.get(hash)
    }

    /// Get the number of entries in the index
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Check if index is empty
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Get all entries
    pub fn entries(&self) -> impl Iterator<Item = &IndexEntry> {
        self.entries.values()
    }

    /// Load index from binary file (Bincode format)
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let data = fs::read(path)?;
        bincode::deserialize(&data)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    }

    /// Save index to binary file (Bincode format)
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
    fn test_add_and_check_entry() {
        let mut index = Index::new();
        let hash = "abc123".to_string();
        let path = "/photos/img1.jpg".to_string();

        index.add_entry(hash.clone(), path.clone());

        assert!(index.contains_hash(&hash));
        assert_eq!(index.len(), 1);
        assert_eq!(index.get_entry(&hash).unwrap().file_path, path);
    }

    #[test]
    fn test_persistence() -> io::Result<()> {
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
}

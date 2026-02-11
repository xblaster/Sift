//! Blake3 hashing module for computing cryptographic hashes of files.
//!
//! This module provides high-performance file hashing using the Blake3 algorithm,
//! optimized for large files with buffered I/O. It supports both individual file
//! hashing and parallel batch processing.
//!
//! # Examples
//!
//! Hash a single file:
//! ```no_run
//! # use sift::hash;
//! let hash = hash::hash_file("image.jpg")?;
//! println!("Hash: {}", hash);
//! # Ok::<(), std::io::Error>(())
//! ```
//!
//! Hash multiple files in parallel:
//! ```no_run
//! # use sift::hash;
//! let paths = vec!["img1.jpg", "img2.jpg", "img3.jpg"];
//! let hashes = hash::hash_files_parallel(paths);
//! for (path, hash) in hashes {
//!     println!("{}: {}", path, hash);
//! }
//! ```

use blake3;
use rayon::prelude::*;
use std::fs::File;
use std::io::{self, Read};
use std::path::Path;

const BLOCK_SIZE: usize = 65536; // 64KB blocks for reading files

/// Computes the Blake3 hash of a file using buffered I/O.
///
/// This function reads a file in 64KB blocks and computes its Blake3 hash.
/// The buffered approach is optimized for files on network shares (SMB/NFS).
///
/// # Arguments
///
/// * `path` - Path to the file to hash
///
/// # Returns
///
/// * `Ok(blake3::Hash)` - The Blake3 hash of the file contents
/// * `Err(io::Error)` - If the file cannot be read
///
/// # Examples
///
/// ```no_run
/// # use sift::hash;
/// let hash = hash::hash_file("photo.jpg")?;
/// assert_eq!(hash.to_hex().len(), 64); // Blake3 produces 64 hex chars
/// # Ok::<(), std::io::Error>(())
/// ```
pub fn hash_file<P: AsRef<Path>>(path: P) -> io::Result<blake3::Hash> {
    let file = File::open(path)?;
    let mut hasher = blake3::Hasher::new();

    let mut reader = io::BufReader::with_capacity(BLOCK_SIZE * 4, file);
    let mut buffer = vec![0; BLOCK_SIZE];

    loop {
        let n = reader.read(&mut buffer)?;
        if n == 0 {
            break;
        }
        hasher.update(&buffer[..n]);
    }

    Ok(hasher.finalize())
}

/// Computes the Blake3 hash of a byte slice.
///
/// # Arguments
///
/// * `data` - Byte slice to hash
///
/// # Returns
///
/// The Blake3 hash of the input data
///
/// # Examples
///
/// ```
/// # use sift::hash;
/// let hash = hash::hash_bytes(b"Hello, world!");
/// assert_eq!(hash.to_hex().len(), 64);
/// ```
pub fn hash_bytes(data: &[u8]) -> blake3::Hash {
    blake3::Hasher::new()
        .update(data)
        .finalize()
}

/// Computes Blake3 hashes for multiple files in parallel using Rayon.
///
/// This function uses Rayon's data parallelism to hash multiple files
/// concurrently. Files that cannot be read are silently skipped.
///
/// # Arguments
///
/// * `paths` - Vector of file paths to hash
///
/// # Returns
///
/// A vector of tuples containing (file_path, hash) for successfully hashed files
///
/// # Examples
///
/// ```no_run
/// # use sift::hash;
/// let paths = vec!["img1.jpg", "img2.jpg"];
/// let results = hash::hash_files_parallel(paths);
/// assert!(results.len() <= 2);
/// ```
pub fn hash_files_parallel<P: AsRef<Path>>(paths: Vec<P>) -> Vec<(String, blake3::Hash)> {
    paths
        .into_iter()
        .map(|p| p.as_ref().to_string_lossy().to_string())
        .collect::<Vec<_>>()
        .into_par_iter()
        .filter_map(|path| {
            match hash_file(&path) {
                Ok(hash) => Some((path, hash)),
                Err(_) => None, // Skip files that can't be read
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_hash_bytes_deterministic() {
        let data = b"Hello, world!";
        let hash1 = hash_bytes(data);
        let hash2 = hash_bytes(data);
        assert_eq!(hash1, hash2, "Same data should produce identical hashes");
    }

    #[test]
    fn test_hash_bytes_different_input() {
        let data1 = b"Hello";
        let data2 = b"World";
        let hash1 = hash_bytes(data1);
        let hash2 = hash_bytes(data2);
        assert_ne!(hash1, hash2, "Different data should produce different hashes");
    }

    #[test]
    fn test_hash_bytes_empty() {
        let empty = b"";
        let hash = hash_bytes(empty);
        assert_eq!(hash.to_hex().len(), 64, "Hash should always be 64 hex chars");
    }

    #[test]
    fn test_hash_bytes_large_data() {
        let large_data = vec![42u8; 1_000_000]; // 1 MB
        let hash = hash_bytes(&large_data);
        assert_eq!(hash.to_hex().len(), 64);
    }

    #[test]
    fn test_hash_file() -> io::Result<()> {
        let mut temp_file = NamedTempFile::new()?;
        temp_file.write_all(b"Test content")?;
        temp_file.flush()?;

        let hash = hash_file(temp_file.path())?;
        let zero_hash = blake3::Hash::from_bytes([0u8; 32]);
        assert_ne!(hash, zero_hash);
        assert_eq!(hash.to_hex().len(), 64);
        Ok(())
    }

    #[test]
    fn test_hash_file_deterministic() -> io::Result<()> {
        let mut temp_file1 = NamedTempFile::new()?;
        temp_file1.write_all(b"Identical content")?;
        temp_file1.flush()?;

        let mut temp_file2 = NamedTempFile::new()?;
        temp_file2.write_all(b"Identical content")?;
        temp_file2.flush()?;

        let hash1 = hash_file(temp_file1.path())?;
        let hash2 = hash_file(temp_file2.path())?;
        assert_eq!(hash1, hash2, "Files with identical content should have identical hashes");
        Ok(())
    }

    #[test]
    fn test_hash_file_nonexistent() {
        let result = hash_file("/nonexistent/path/file.jpg");
        assert!(result.is_err(), "Should return error for nonexistent file");
    }

    #[test]
    fn test_hash_file_large() -> io::Result<()> {
        let mut temp_file = NamedTempFile::new()?;
        let large_data = vec![42u8; 10_000_000]; // 10 MB
        temp_file.write_all(&large_data)?;
        temp_file.flush()?;

        let hash = hash_file(temp_file.path())?;
        assert_eq!(hash.to_hex().len(), 64);
        Ok(())
    }

    #[test]
    fn test_hash_files_parallel_empty() {
        let paths: Vec<String> = vec![];
        let results = hash_files_parallel(paths);
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn test_hash_files_parallel() -> io::Result<()> {
        let mut file1 = NamedTempFile::new()?;
        file1.write_all(b"Content 1")?;
        file1.flush()?;

        let mut file2 = NamedTempFile::new()?;
        file2.write_all(b"Content 2")?;
        file2.flush()?;

        let paths = vec![file1.path().to_path_buf(), file2.path().to_path_buf()];
        let results = hash_files_parallel(paths);
        assert_eq!(results.len(), 2, "Should hash both files");
        assert_ne!(results[0].1, results[1].1, "Different files should have different hashes");
        Ok(())
    }

    #[test]
    fn test_hash_files_parallel_with_missing() -> io::Result<()> {
        let mut valid_file = NamedTempFile::new()?;
        valid_file.write_all(b"Valid content")?;
        valid_file.flush()?;

        let paths = vec![
            valid_file.path().to_path_buf(),
            "/nonexistent/file.jpg".into(),
        ];
        let results = hash_files_parallel(paths);
        assert_eq!(results.len(), 1, "Should skip nonexistent files");
        Ok(())
    }
}

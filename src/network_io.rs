//! Optimized I/O operations for network storage (SMB/NFS).
//!
//! This module provides functions for reading files from network shares with
//! optimized buffering and retry logic to handle network latency and temporary
//! connection issues.
//!
//! # Features
//!
//! * 1MB buffered reads for optimal throughput on network shares
//! * Exponential backoff retry mechanism for transient failures
//! * Support for reading specific file chunks
//!
//! # Examples
//!
//! Read a file with automatic retries:
//! ```no_run
//! # use sift::network_io;
//! let data = network_io::read_file_with_retries("/mnt/smb/photo.jpg")?;
//! println!("Read {} bytes", data.len());
//! # Ok::<(), std::io::Error>(())
//! ```

use std::fs::File;
use std::io::{self, BufReader, Read, Seek};
use std::path::Path;
use std::thread;
use std::time::Duration;

const BUFFER_SIZE: usize = 1_048_576; // 1 MB buffer for network reads
const MAX_RETRIES: usize = 3;
const INITIAL_RETRY_DELAY_MS: u64 = 100;

/// Reads a file with optimized buffering for network shares (SMB/NFS).
///
/// Uses a 1MB buffer to efficiently read large files from network storage,
/// minimizing the number of network round-trips required.
///
/// # Arguments
///
/// * `path` - Path to the file to read
///
/// # Returns
///
/// * `Ok(Vec<u8>)` - The file contents
/// * `Err(io::Error)` - If the file cannot be read
///
/// # Examples
///
/// ```no_run
/// # use sift::network_io;
/// let data = network_io::buffered_read_file("/mnt/smb/photo.jpg")?;
/// println!("Read {} bytes", data.len());
/// # Ok::<(), std::io::Error>(())
/// ```
pub fn buffered_read_file<P: AsRef<Path>>(path: P) -> io::Result<Vec<u8>> {
    let file = File::open(path)?;
    let mut reader = BufReader::with_capacity(BUFFER_SIZE, file);
    let mut data = Vec::new();
    reader.read_to_end(&mut data)?;
    Ok(data)
}

/// Reads a file with automatic retry logic for transient network errors.
///
/// Uses exponential backoff to retry failed read attempts. After MAX_RETRIES
/// consecutive failures, the last error is returned to the caller.
///
/// # Arguments
///
/// * `path` - Path to the file to read
///
/// # Returns
///
/// * `Ok(Vec<u8>)` - The file contents (possibly after retries)
/// * `Err(io::Error)` - If all retry attempts fail
///
/// # Retry Behavior
///
/// * First retry: 100ms delay
/// * Second retry: 200ms delay
/// * Third retry: 400ms delay
/// * Gives up after 3 failures
///
/// # Examples
///
/// ```no_run
/// # use sift::network_io;
/// // This might succeed even if the network is temporarily unavailable
/// match network_io::read_file_with_retries("/mnt/smb/photo.jpg") {
///     Ok(data) => println!("Successfully read {} bytes", data.len()),
///     Err(e) => println!("Failed after retries: {}", e),
/// }
/// ```
pub fn read_file_with_retries<P: AsRef<Path>>(path: P) -> io::Result<Vec<u8>> {
    read_with_exponential_backoff(|| buffered_read_file(&path))
}

/// Generic retry function with exponential backoff for any I/O operation.
///
/// Implements exponential backoff retry logic for resilience against
/// transient network failures.
fn read_with_exponential_backoff<F>(mut operation: F) -> io::Result<Vec<u8>>
where
    F: FnMut() -> io::Result<Vec<u8>>,
{
    let mut last_error = None;
    let mut delay_ms = INITIAL_RETRY_DELAY_MS;

    for attempt in 0..=MAX_RETRIES {
        match operation() {
            Ok(data) => {
                if attempt > 0 {
                    eprintln!("Successfully read after {} retries", attempt);
                }
                return Ok(data);
            }
            Err(e) => {
                last_error = Some(e);

                if attempt < MAX_RETRIES {
                    eprintln!(
                        "Read attempt {} failed, retrying in {}ms...",
                        attempt + 1,
                        delay_ms
                    );
                    thread::sleep(Duration::from_millis(delay_ms));
                    delay_ms *= 2; // Exponential backoff
                }
            }
        }
    }

    Err(last_error.unwrap_or_else(|| {
        io::Error::other("Unknown error after retries")
    }))
}

/// Reads a specific chunk (range) from a file.
///
/// Useful for reading parts of large files without loading the entire file into memory.
/// Seeks to the specified offset and reads up to `size` bytes.
///
/// # Arguments
///
/// * `path` - Path to the file
/// * `offset` - Byte offset to start reading from
/// * `size` - Number of bytes to read
///
/// # Returns
///
/// * `Ok(Vec<u8>)` - The file chunk (may be smaller than `size` if EOF is reached)
/// * `Err(io::Error)` - If the file cannot be read or seeked
///
/// # Examples
///
/// ```no_run
/// # use sift::network_io;
/// // Read first 1MB of a file
/// let chunk = network_io::read_file_chunk("large_file.jpg", 0, 1_048_576)?;
/// println!("Read {} bytes", chunk.len());
/// # Ok::<(), std::io::Error>(())
/// ```
pub fn read_file_chunk<P: AsRef<Path>>(
    path: P,
    offset: u64,
    size: usize,
) -> io::Result<Vec<u8>> {
    let mut file = File::open(path)?;
    file.seek(std::io::SeekFrom::Start(offset))?;

    let mut reader = BufReader::with_capacity(BUFFER_SIZE, file);
    let mut buffer = vec![0; size];
    let bytes_read = reader.read(&mut buffer)?;
    buffer.truncate(bytes_read);

    Ok(buffer)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_buffered_read_file_small() -> io::Result<()> {
        let mut temp_file = NamedTempFile::new()?;
        let test_data = b"Hello, world! This is a test file.";
        temp_file.write_all(test_data)?;
        temp_file.flush()?;

        let data = buffered_read_file(temp_file.path())?;
        assert_eq!(data, test_data);

        Ok(())
    }

    #[test]
    fn test_buffered_read_file_empty() -> io::Result<()> {
        let temp_file = NamedTempFile::new()?;
        let data = buffered_read_file(temp_file.path())?;
        assert!(data.is_empty());

        Ok(())
    }

    #[test]
    fn test_buffered_read_file_large() -> io::Result<()> {
        let mut temp_file = NamedTempFile::new()?;
        let large_data = vec![42u8; 5_000_000]; // 5 MB
        temp_file.write_all(&large_data)?;
        temp_file.flush()?;

        let data = buffered_read_file(temp_file.path())?;
        assert_eq!(data.len(), large_data.len());
        assert_eq!(data, large_data);

        Ok(())
    }

    #[test]
    fn test_buffered_read_file_nonexistent() {
        let result = buffered_read_file("/nonexistent/path/file.jpg");
        assert!(result.is_err());
    }

    #[test]
    fn test_read_file_with_retries_success() -> io::Result<()> {
        let mut temp_file = NamedTempFile::new()?;
        let test_data = b"Test data for retry";
        temp_file.write_all(test_data)?;
        temp_file.flush()?;

        let data = read_file_with_retries(temp_file.path())?;
        assert_eq!(data, test_data);

        Ok(())
    }

    #[test]
    fn test_read_file_with_retries_nonexistent() {
        let result = read_file_with_retries("/nonexistent/path/file.jpg");
        assert!(result.is_err());
    }

    #[test]
    fn test_read_file_chunk() -> io::Result<()> {
        let mut temp_file = NamedTempFile::new()?;
        let test_data = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ";
        temp_file.write_all(test_data)?;
        temp_file.flush()?;

        // Read first 5 bytes
        let chunk = read_file_chunk(temp_file.path(), 0, 5)?;
        assert_eq!(chunk, b"01234");

        Ok(())
    }

    #[test]
    fn test_read_file_chunk_offset() -> io::Result<()> {
        let mut temp_file = NamedTempFile::new()?;
        let test_data = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ";
        temp_file.write_all(test_data)?;
        temp_file.flush()?;

        // Read 5 bytes starting at offset 10
        let chunk = read_file_chunk(temp_file.path(), 10, 5)?;
        assert_eq!(chunk, b"ABCDE");

        Ok(())
    }

    #[test]
    fn test_read_file_chunk_beyond_eof() -> io::Result<()> {
        let mut temp_file = NamedTempFile::new()?;
        let test_data = b"Hello";
        temp_file.write_all(test_data)?;
        temp_file.flush()?;

        // Try to read beyond file size
        let chunk = read_file_chunk(temp_file.path(), 2, 100)?;
        assert_eq!(chunk, b"llo");

        Ok(())
    }

    #[test]
    fn test_read_file_chunk_at_eof() -> io::Result<()> {
        let mut temp_file = NamedTempFile::new()?;
        let test_data = b"Hello";
        temp_file.write_all(test_data)?;
        temp_file.flush()?;

        // Read at exact EOF
        let chunk = read_file_chunk(temp_file.path(), 5, 10)?;
        assert!(chunk.is_empty());

        Ok(())
    }

    #[test]
    fn test_read_file_chunk_nonexistent() {
        let result = read_file_chunk("/nonexistent/path/file.jpg", 0, 100);
        assert!(result.is_err());
    }
}

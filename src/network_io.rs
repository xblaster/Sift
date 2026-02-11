use std::fs::File;
use std::io::{self, BufReader, Read, Seek};
use std::path::Path;
use std::thread;
use std::time::Duration;

const BUFFER_SIZE: usize = 1_048_576; // 1 MB buffer for network reads
const MAX_RETRIES: usize = 3;
const INITIAL_RETRY_DELAY_MS: u64 = 100;

/// Read a file with optimized buffering for network shares (SMB/NFS)
pub fn buffered_read_file<P: AsRef<Path>>(path: P) -> io::Result<Vec<u8>> {
    let file = File::open(path)?;
    let mut reader = BufReader::with_capacity(BUFFER_SIZE, file);
    let mut data = Vec::new();
    reader.read_to_end(&mut data)?;
    Ok(data)
}

/// Read a file with retry logic for network errors
pub fn read_file_with_retries<P: AsRef<Path>>(path: P) -> io::Result<Vec<u8>> {
    read_with_exponential_backoff(|| buffered_read_file(&path))
}

/// Generic retry function with exponential backoff
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
        io::Error::new(io::ErrorKind::Other, "Unknown error after retries")
    }))
}

/// Read a specific chunk of a file (for large files)
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
    fn test_buffered_read_file() -> io::Result<()> {
        let mut temp_file = NamedTempFile::new()?;
        let test_data = b"Hello, world! This is a test file.";
        temp_file.write_all(test_data)?;
        temp_file.flush()?;

        let data = buffered_read_file(temp_file.path())?;
        assert_eq!(data, test_data);

        Ok(())
    }

    #[test]
    fn test_read_file_with_retries() -> io::Result<()> {
        let mut temp_file = NamedTempFile::new()?;
        let test_data = b"Test data for retry";
        temp_file.write_all(test_data)?;
        temp_file.flush()?;

        let data = read_file_with_retries(temp_file.path())?;
        assert_eq!(data, test_data);

        Ok(())
    }
}

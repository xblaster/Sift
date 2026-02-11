use blake3;
use rayon::prelude::*;
use std::fs::File;
use std::io::{self, Read};
use std::path::Path;

const BLOCK_SIZE: usize = 65536; // 64KB blocks for reading files

/// Compute Blake3 hash of a file using parallelized reading
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

/// Compute Blake3 hash of raw bytes
pub fn hash_bytes(data: &[u8]) -> blake3::Hash {
    blake3::Hasher::new()
        .update(data)
        .finalize()
}

/// Compute Blake3 hash for multiple files in parallel
pub fn hash_files_parallel<P: AsRef<Path>>(paths: Vec<P>) -> Vec<(String, blake3::Hash)> {
    paths
        .into_par_iter()
        .filter_map(|path| {
            let path_ref = path.as_ref();
            match hash_file(path_ref) {
                Ok(hash) => Some((
                    path_ref.to_string_lossy().to_string(),
                    hash,
                )),
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
    fn test_hash_bytes() {
        let data = b"Hello, world!";
        let hash1 = hash_bytes(data);
        let hash2 = hash_bytes(data);
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_hash_file() -> io::Result<()> {
        let mut temp_file = NamedTempFile::new()?;
        temp_file.write_all(b"Test content")?;
        temp_file.flush()?;

        let hash = hash_file(temp_file.path())?;
        assert_ne!(hash, blake3::Hash::default());
        Ok(())
    }
}

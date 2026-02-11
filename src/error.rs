//! Error types for Sift photo organization.

use std::fmt;
use std::io;

/// Errors that can occur during photo organization.
#[derive(Debug)]
pub enum OrganizeError {
    /// I/O operation failed
    IoError(io::Error),
    /// File access error (permission denied, file not found)
    FileAccess(String),
    /// Metadata extraction failed
    MetadataError(String),
    /// Hash computation failed
    HashError(String),
    /// Index corruption or loading error
    IndexError(String),
    /// Organization/copying failed
    OrganizationError(String),
    /// Network error (for SMB/NFS operations)
    NetworkError(String),
    /// Clustering error
    ClusteringError(String),
    /// Generic error with message
    Other(String),
}

impl fmt::Display for OrganizeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OrganizeError::IoError(e) => write!(f, "I/O error: {}", e),
            OrganizeError::FileAccess(msg) => write!(f, "File access error: {}", msg),
            OrganizeError::MetadataError(msg) => write!(f, "Metadata error: {}", msg),
            OrganizeError::HashError(msg) => write!(f, "Hash error: {}", msg),
            OrganizeError::IndexError(msg) => write!(f, "Index error: {}", msg),
            OrganizeError::OrganizationError(msg) => write!(f, "Organization error: {}", msg),
            OrganizeError::NetworkError(msg) => write!(f, "Network error: {}", msg),
            OrganizeError::ClusteringError(msg) => write!(f, "Clustering error: {}", msg),
            OrganizeError::Other(msg) => write!(f, "Error: {}", msg),
        }
    }
}

impl std::error::Error for OrganizeError {}

impl From<io::Error> for OrganizeError {
    fn from(err: io::Error) -> Self {
        OrganizeError::IoError(err)
    }
}

/// Result type for operations that can fail with `OrganizeError`.
pub type OrganizeResult<T> = Result<T, OrganizeError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = OrganizeError::MetadataError("No date found".to_string());
        assert!(err.to_string().contains("Metadata error"));
    }

    #[test]
    fn test_error_from_io() {
        let io_err = io::Error::new(io::ErrorKind::NotFound, "file not found");
        let err: OrganizeError = io_err.into();
        assert!(err.to_string().contains("I/O error"));
    }

    #[test]
    fn test_error_display_variants() {
        let errors = vec![
            (OrganizeError::FileAccess("denied".to_string()), "File access"),
            (OrganizeError::HashError("bad data".to_string()), "Hash error"),
            (OrganizeError::IndexError("corrupt".to_string()), "Index error"),
            (OrganizeError::OrganizationError("copy failed".to_string()), "Organization"),
            (OrganizeError::NetworkError("timeout".to_string()), "Network"),
        ];

        for (err, expected) in errors {
            assert!(err.to_string().contains(expected));
        }
    }
}

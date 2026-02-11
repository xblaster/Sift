//! Photo metadata extraction from EXIF data and file attributes.
//!
//! This module provides functionality to extract temporal metadata from photos,
//! prioritizing EXIF DateTimeOriginal and falling back to file modification time.
//! It also provides utilities for organizing files chronologically.
//!
//! # Examples
//!
//! Extract the date from a photo:
//! ```no_run
//! # use sift::metadata;
//! let date = metadata::extract_date("photo.jpg")?;
//! println!("Photo taken: {}", date);
//! # Ok::<(), std::io::Error>(())
//! ```

use chrono::{DateTime, Local, NaiveDate};
use exif::Reader;
use std::fs;
use std::io;
use std::path::Path;
use std::time::SystemTime;

/// Metadata extracted from a photo file.
///
/// # Fields
///
/// * `file_path` - The original path to the photo file
/// * `date_taken` - The date the photo was taken (from EXIF or mtime)
#[derive(Debug, Clone)]
pub struct PhotoMetadata {
    pub file_path: String,
    pub date_taken: NaiveDate,
}

/// Extracts the date taken from a photo file.
///
/// This function attempts to read the EXIF DateTimeOriginal tag first.
/// If EXIF data is not available, it falls back to the file's modification time (mtime).
///
/// # Arguments
///
/// * `path` - Path to the photo file
///
/// # Returns
///
/// * `Ok(NaiveDate)` - The date the photo was taken
/// * `Err(io::Error)` - If the file cannot be accessed
///
/// # Examples
///
/// ```no_run
/// # use sift::metadata;
/// let date = metadata::extract_date("photo.jpg")?;
/// println!("Taken on: {}", date);
/// # Ok::<(), std::io::Error>(())
/// ```
pub fn extract_date<P: AsRef<Path>>(path: P) -> io::Result<NaiveDate> {
    let path_ref = path.as_ref();

    // Try to read EXIF data first
    if let Ok(file) = fs::File::open(path_ref) {
        if let Ok(reader) = Reader::new().read_from_buffer(&mut io::BufReader::new(file)) {
            // Look for DateTimeOriginal tag
            if let Some(date_field) = reader.get_field(exif::Tag::DateTime, exif::In::Primary) {
                if let Ok(date_str) = date_field.display_value().to_string().parse::<String>() {
                    // Parse EXIF DateTime format: "YYYY:MM:DD HH:MM:SS"
                    if let Ok(date) = NaiveDate::parse_from_str(&date_str[..10], "%Y:%m:%d") {
                        return Ok(date);
                    }
                }
            }
        }
    }

    // Fallback to file modification time
    let metadata = fs::metadata(path_ref)?;
    let modified = metadata.modified()?;

    let datetime: DateTime<Local> = modified.into();
    Ok(datetime.naive_local().date())
}

/// Extracts the date taken from a photo file, returning `None` on error.
///
/// This is a safe wrapper around `extract_date` that returns `None` instead of
/// propagating errors. Useful for batch operations where you want to skip files
/// that can't be read.
///
/// # Arguments
///
/// * `path` - Path to the photo file
///
/// # Returns
///
/// * `Some(NaiveDate)` - The date the photo was taken
/// * `None` - If the date cannot be extracted
///
/// # Examples
///
/// ```no_run
/// # use sift::metadata;
/// if let Some(date) = metadata::extract_date_safe("photo.jpg") {
///     println!("Photo taken: {}", date);
/// } else {
///     println!("Could not extract date");
/// }
/// ```
pub fn extract_date_safe<P: AsRef<Path>>(path: P) -> Option<NaiveDate> {
    extract_date(path).ok()
}

/// Builds a chronological folder path from a date.
///
/// Creates a path string in the format `YYYY/MM/DD` suitable for organizing
/// files into date-based directory structures.
///
/// # Arguments
///
/// * `date` - The date to convert
///
/// # Returns
///
/// A string in the format `YYYY/MM/DD`
///
/// # Examples
///
/// ```
/// # use sift::metadata;
/// # use chrono::NaiveDate;
/// let date = NaiveDate::from_ymd_opt(2023, 10, 15).unwrap();
/// let path = metadata::build_chronological_path(date);
/// assert_eq!(path, "2023/10/15");
/// ```
pub fn build_chronological_path(date: NaiveDate) -> String {
    format!(
        "{}/{:02}/{:02}",
        date.year(),
        date.month(),
        date.day()
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_build_chronological_path() {
        let date = NaiveDate::from_ymd_opt(2023, 10, 15).unwrap();
        let path = build_chronological_path(date);
        assert_eq!(path, "2023/10/15");
    }

    #[test]
    fn test_build_chronological_path_january() {
        let date = NaiveDate::from_ymd_opt(2024, 1, 5).unwrap();
        let path = build_chronological_path(date);
        assert_eq!(path, "2024/01/05");
    }

    #[test]
    fn test_build_chronological_path_december() {
        let date = NaiveDate::from_ymd_opt(2024, 12, 31).unwrap();
        let path = build_chronological_path(date);
        assert_eq!(path, "2024/12/31");
    }

    #[test]
    fn test_build_chronological_path_padding() {
        // Ensure month and day are zero-padded
        let date = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        let path = build_chronological_path(date);
        assert_eq!(path, "2024/01/01");
        assert!(path.contains("/01/"));
    }

    #[test]
    fn test_extract_date_from_mtime() -> io::Result<()> {
        let mut temp_file = NamedTempFile::new()?;
        temp_file.write_all(b"Test")?;
        temp_file.flush()?;

        let date = extract_date(temp_file.path())?;
        let now = Local::now().naive_local().date();
        assert!(date <= now, "Extracted date should not be in the future");
        Ok(())
    }

    #[test]
    fn test_extract_date_safe_valid_file() -> io::Result<()> {
        let mut temp_file = NamedTempFile::new()?;
        temp_file.write_all(b"Test")?;
        temp_file.flush()?;

        let date = extract_date_safe(temp_file.path());
        assert!(date.is_some(), "Should extract date from valid file");
        Ok(())
    }

    #[test]
    fn test_extract_date_safe_missing_file() {
        let date = extract_date_safe("/nonexistent/path/file.jpg");
        assert!(date.is_none(), "Should return None for missing file");
    }

    #[test]
    fn test_extract_date_nonexistent_file() {
        let result = extract_date("/nonexistent/path/file.jpg");
        assert!(result.is_err(), "Should return error for nonexistent file");
    }

    #[test]
    fn test_extract_date_multiple_files() -> io::Result<()> {
        let mut file1 = NamedTempFile::new()?;
        file1.write_all(b"File 1")?;
        file1.flush()?;

        let mut file2 = NamedTempFile::new()?;
        file2.write_all(b"File 2")?;
        file2.flush()?;

        let date1 = extract_date(file1.path())?;
        let date2 = extract_date(file2.path())?;

        assert_eq!(date1, date2, "Files created at same time should have same date");
        Ok(())
    }

    #[test]
    fn test_photo_metadata_creation() {
        let metadata = PhotoMetadata {
            file_path: "/photos/img.jpg".to_string(),
            date_taken: NaiveDate::from_ymd_opt(2023, 10, 15).unwrap(),
        };

        assert_eq!(metadata.file_path, "/photos/img.jpg");
        assert_eq!(metadata.date_taken.year(), 2023);
        assert_eq!(metadata.date_taken.month(), 10);
        assert_eq!(metadata.date_taken.day(), 15);
    }

    #[test]
    fn test_build_chronological_path_range() {
        // Test a range of dates
        for month in 1..=12 {
            let date = NaiveDate::from_ymd_opt(2024, month, 15).unwrap();
            let path = build_chronological_path(date);
            assert!(path.contains("2024"));
            assert!(path.contains(&format!("/{:02}/", month)));
        }
    }
}

//! Photo metadata extraction from file attributes.
//!
//! This module provides functionality to extract temporal metadata from photos
//! using file modification time. It also provides utilities for organizing files
//! chronologically.
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

use chrono::{DateTime, Local, NaiveDate, Datelike};
use exif::{In, Tag};
use std::fs;
use std::io;
use std::path::Path;

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

/// Extracts the date taken from a photo file's EXIF data.
///
/// Priority is given to the `DateTimeOriginal` tag.
///
/// # Arguments
///
/// * `path` - Path to the photo file
///
/// # Returns
///
/// * `Some(NaiveDate)` - The extracted date if found and valid
/// * `None` - If EXIF data is missing or doesn't contain a valid date
pub fn extract_exif_date<P: AsRef<Path>>(path: P) -> Option<NaiveDate> {
    let file = fs::File::open(path).ok()?;
    let mut reader = io::BufReader::new(file);
    let exifreader = exif::Reader::new();
    let exif = exifreader.read_from_container(&mut reader).ok()?;

    if let Some(field) = exif.get_field(Tag::DateTimeOriginal, In::PRIMARY) {
        let value = format!("{}", field.display_value());
        // EXIF date format is usually "YYYY:MM:DD HH:MM:SS"
        if value.len() >= 10 {
            let year = value[0..4].parse::<i32>().ok()?;
            let month = value[5..7].parse::<u32>().ok()?;
            let day = value[8..10].parse::<u32>().ok()?;
            return NaiveDate::from_ymd_opt(year, month, day);
        }
    }
    None
}

/// Extracts the date taken from a photo file.
///
/// This function uses the file's modification time (mtime) as the source for date extraction.
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

    // Extract date from file modification time
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

/// Extracts the date from a filename using YYYYMMDD pattern.
///
/// Attempts to parse a filename for a date in YYYYMMDD format.
/// For example, "IMG_20240211.jpg" would return 2024-02-11.
///
/// # Arguments
///
/// * `filename` - The filename to parse (without path)
///
/// # Returns
///
/// * `Some(NaiveDate)` - If a valid YYYYMMDD pattern is found
/// * `None` - If no valid date pattern is found
///
/// # Examples
///
/// ```
/// # use sift::metadata;
/// let date = metadata::extract_date_from_filename("IMG_20240211_001.jpg");
/// assert!(date.is_some());
/// ```
pub fn extract_date_from_filename(filename: &str) -> Option<NaiveDate> {
    // Look for YYYYMMDD pattern in filename
    for i in 0..filename.len().saturating_sub(7) {
        if let Ok(date_str) = &filename[i..i + 8].parse::<String>()
            && date_str.chars().all(|c| c.is_ascii_digit())
                && let Ok(year) = date_str[0..4].parse::<i32>()
                    && let Ok(month) = date_str[4..6].parse::<u32>()
                        && let Ok(day) = date_str[6..8].parse::<u32>()
                            && (2000..=2100).contains(&year) && (1..=12).contains(&month) && (1..=31).contains(&day) {
                                return NaiveDate::from_ymd_opt(year, month, day);
                            }
    }
    None
}

/// Extracts date using a priority-based fallback strategy.
///
/// Attempts to extract the date from a photo file using the following priority:
/// 1. EXIF metadata (DateTimeOriginal)
/// 2. Filename pattern (YYYYMMDD format)
/// 3. File modification time (mtime)
///
/// This function provides a best-effort approach to finding the most accurate
/// capture date for a photo file.
///
/// # Arguments
///
/// * `path` - Path to the photo file
///
/// # Returns
///
/// * `Some(NaiveDate)` - The extracted date
/// * `None` - If the date cannot be extracted by any method
pub fn extract_date_with_fallback<P: AsRef<Path>>(path: P) -> Option<NaiveDate> {
    let path_ref = path.as_ref();

    // 1. Try EXIF
    if let Some(date) = extract_exif_date(path_ref) {
        return Some(date);
    }

    // 2. Try to extract from filename
    if let Some(filename) = path_ref.file_name()
        && let Some(filename_str) = filename.to_str()
            && let Some(date) = extract_date_from_filename(filename_str) {
                return Some(date);
            }

    // 3. Fallback to file modification time
    extract_date_safe(path_ref)
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

    #[test]
    fn test_extract_date_from_filename_valid() {
        let date = extract_date_from_filename("IMG_20240211_001.jpg");
        assert!(date.is_some());
        let d = date.unwrap();
        assert_eq!(d.year(), 2024);
        assert_eq!(d.month(), 2);
        assert_eq!(d.day(), 11);
    }

    #[test]
    fn test_extract_date_from_filename_no_date() {
        let date = extract_date_from_filename("random_photo.jpg");
        assert!(date.is_none());
    }

    #[test]
    fn test_extract_date_from_filename_various_patterns() {
        assert!(extract_date_from_filename("photo_20200101.jpg").is_some());
        assert!(extract_date_from_filename("IMG_20231231_123.raw").is_some());
        assert!(extract_date_from_filename("20240615_test.png").is_some());
        assert_eq!(
            extract_date_from_filename("20240211.jpg"),
            Some(NaiveDate::from_ymd_opt(2024, 2, 11).unwrap())
        );
    }

    #[test]
    fn test_extract_date_from_filename_invalid_dates() {
        // Invalid month
        assert!(extract_date_from_filename("photo_20241301.jpg").is_none());
        // Invalid day
        assert!(extract_date_from_filename("photo_20240232.jpg").is_none());
        // Year out of range
        assert!(extract_date_from_filename("photo_19900101.jpg").is_none());
    }

    #[test]
    fn test_extract_date_with_fallback_filename_priority() {
        // Even if the file doesn't exist, if the filename has a date, it should be used
        let path = Path::new("IMG_20200101_999.jpg");
        let date = extract_date_with_fallback(path);
        assert_eq!(date, Some(NaiveDate::from_ymd_opt(2020, 1, 1).unwrap()));
    }

    #[test]
    fn test_extract_date_with_fallback_mtime_fallback() -> io::Result<()> {
        let mut temp_file = NamedTempFile::new()?;
        temp_file.write_all(b"test")?;
        temp_file.flush()?;

        // File name has no date
        let date = extract_date_with_fallback(temp_file.path());
        assert!(date.is_some());
        // Should be today's date (or whenever the file was created in the test)
        let now = Local::now().naive_local().date();
        assert_eq!(date.unwrap(), now);
        Ok(())
    }
}

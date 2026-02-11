//! File organization and folder structure management.
//!
//! This module provides functions to organize photos into folder hierarchies
//! based on capture dates and geographic locations. It handles creating the
//! necessary directory structure and copying files to their final locations.
//!
//! # Examples
//!
//! Organize a photo by date:
//! ```no_run
//! # use sift::organization;
//! # use chrono::NaiveDate;
//! let date = NaiveDate::from_ymd_opt(2023, 10, 15).unwrap();
//! let dest = organization::organize_by_date(
//!     "source.jpg",
//!     "/photos",
//!     date
//! )?;
//! println!("Organized to: {:?}", dest);
//! # Ok::<(), std::io::Error>(())
//! ```

use chrono::{NaiveDate, Datelike};
use std::fs;
use std::path::{Path, PathBuf};
use std::io;

/// Organizes a file into a chronological folder structure (YYYY/MM/DD).
///
/// Creates the necessary directory structure and copies the file to the destination.
/// The file is placed in a subfolder hierarchy based on its capture date.
///
/// # Arguments
///
/// * `source_file` - Path to the source file
/// * `dest_root` - Root destination directory
/// * `date` - The date to use for folder organization
///
/// # Returns
///
/// * `Ok(PathBuf)` - Path to the copied file in the destination
/// * `Err(io::Error)` - If the operation fails
///
/// # Examples
///
/// ```no_run
/// # use sift::organization;
/// # use chrono::NaiveDate;
/// let date = NaiveDate::from_ymd_opt(2023, 10, 15).unwrap();
/// let result = organization::organize_by_date(
///     "photo.jpg",
///     "/organized_photos",
///     date
/// )?;
/// assert!(result.exists());
/// # Ok::<(), std::io::Error>(())
/// ```
pub fn organize_by_date<P: AsRef<Path>>(
    source_file: P,
    dest_root: P,
    date: NaiveDate,
) -> io::Result<PathBuf> {
    let source = source_file.as_ref();
    let root = dest_root.as_ref();

    // Build destination path
    let chrono_path = format!(
        "{}/{:02}/{:02}",
        date.year(),
        date.month(),
        date.day()
    );
    let dest_dir = root.join(&chrono_path);

    // Create folder structure
    fs::create_dir_all(&dest_dir)?;

    // Copy or move file
    let file_name = source
        .file_name()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "Invalid file name"))?;

    let dest_file = dest_dir.join(file_name);

    // Copy file (not move, to preserve source)
    fs::copy(source, &dest_file)?;

    Ok(dest_file)
}

/// Organizes a file into a chronological folder structure with geographic location.
///
/// Creates a directory structure combining both chronological organization
/// (YYYY/MM/DD) and geographic clustering (by location name).
/// This is useful for organizing clustered photos geographically.
///
/// # Arguments
///
/// * `source_file` - Path to the source file
/// * `dest_root` - Root destination directory
/// * `date` - The date to use for folder organization
/// * `location` - The location name (e.g., "Paris", "New York")
///
/// # Returns
///
/// * `Ok(PathBuf)` - Path to the copied file in the destination
/// * `Err(io::Error)` - If the operation fails
///
/// # Examples
///
/// ```no_run
/// # use sift::organization;
/// # use chrono::NaiveDate;
/// let date = NaiveDate::from_ymd_opt(2023, 10, 15).unwrap();
/// let result = organization::organize_by_date_and_location(
///     "photo.jpg",
///     "/organized_photos",
///     date,
///     "Paris"
/// )?;
/// // File will be at: /organized_photos/2023/10/15/Paris/photo.jpg
/// # Ok::<(), std::io::Error>(())
/// ```
pub fn organize_by_date_and_location<P: AsRef<Path>>(
    source_file: P,
    dest_root: P,
    date: NaiveDate,
    location: &str,
) -> io::Result<PathBuf> {
    let source = source_file.as_ref();
    let root = dest_root.as_ref();

    // Build destination path with location subfolder
    let chrono_path = format!(
        "{}/{:02}/{:02}/{}",
        date.year(),
        date.month(),
        date.day(),
        location
    );
    let dest_dir = root.join(&chrono_path);

    // Create folder structure
    fs::create_dir_all(&dest_dir)?;

    // Copy file
    let file_name = source
        .file_name()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "Invalid file name"))?;

    let dest_file = dest_dir.join(file_name);
    fs::copy(source, &dest_file)?;

    Ok(dest_file)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::{tempdir, NamedTempFile};

    #[test]
    fn test_organize_by_date_basic() -> io::Result<()> {
        let source_dir = tempdir()?;
        let dest_dir = tempdir()?;

        let mut source_file = NamedTempFile::new_in(source_dir.path())?;
        source_file.write_all(b"Test image")?;
        source_file.flush()?;

        let date = NaiveDate::from_ymd_opt(2023, 10, 15).unwrap();
        let result = organize_by_date(source_file.path(), dest_dir.path(), date)?;

        assert!(result.exists());
        assert!(result.to_string_lossy().contains("2023/10/15"));
        assert!(result.to_string_lossy().ends_with(source_file.path().file_name().unwrap().to_str().unwrap()));

        Ok(())
    }

    #[test]
    fn test_organize_by_date_creates_hierarchy() -> io::Result<()> {
        let dest_dir = tempdir()?;
        let mut source_file = NamedTempFile::new()?;
        source_file.write_all(b"Test")?;
        source_file.flush()?;

        let date = NaiveDate::from_ymd_opt(2024, 12, 25).unwrap();
        let result = organize_by_date(source_file.path(), dest_dir.path(), date)?;

        // Check that all parent directories were created
        assert!(result.parent().unwrap().exists());
        assert!(result.parent().unwrap().parent().unwrap().exists());

        Ok(())
    }

    #[test]
    fn test_organize_by_date_copies_content() -> io::Result<()> {
        let source_dir = tempdir()?;
        let dest_dir = tempdir()?;

        let test_content = b"Unique test content for this file";
        let mut source_file = NamedTempFile::new_in(source_dir.path())?;
        source_file.write_all(test_content)?;
        source_file.flush()?;

        let date = NaiveDate::from_ymd_opt(2023, 6, 20).unwrap();
        let result = organize_by_date(source_file.path(), dest_dir.path(), date)?;

        let copied_content = fs::read(&result)?;
        assert_eq!(copied_content, test_content);

        Ok(())
    }

    #[test]
    fn test_organize_by_date_and_location_basic() -> io::Result<()> {
        let source_dir = tempdir()?;
        let dest_dir = tempdir()?;

        let mut source_file = NamedTempFile::new_in(source_dir.path())?;
        source_file.write_all(b"Test image")?;
        source_file.flush()?;

        let date = NaiveDate::from_ymd_opt(2023, 10, 15).unwrap();
        let result = organize_by_date_and_location(
            source_file.path(),
            dest_dir.path(),
            date,
            "Paris",
        )?;

        assert!(result.exists());
        assert!(result.to_string_lossy().contains("2023/10/15/Paris"));

        Ok(())
    }

    #[test]
    fn test_organize_by_date_and_location_multiple_locations() -> io::Result<()> {
        let source_dir = tempdir()?;
        let dest_dir = tempdir()?;

        let locations = vec!["Paris", "London", "Berlin"];
        let date = NaiveDate::from_ymd_opt(2023, 5, 10).unwrap();

        for location in locations {
            let mut source_file = NamedTempFile::new_in(source_dir.path())?;
            source_file.write_all(b"Test")?;
            source_file.flush()?;

            let result = organize_by_date_and_location(
                source_file.path(),
                dest_dir.path(),
                date,
                location,
            )?;

            assert!(result.to_string_lossy().contains(location));
        }

        Ok(())
    }

    #[test]
    fn test_organize_by_date_january() -> io::Result<()> {
        let source_dir = tempdir()?;
        let dest_dir = tempdir()?;

        let mut source_file = NamedTempFile::new_in(source_dir.path())?;
        source_file.write_all(b"Test")?;
        source_file.flush()?;

        let date = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        let result = organize_by_date(source_file.path(), dest_dir.path(), date)?;

        assert!(result.to_string_lossy().contains("2024/01/01"));

        Ok(())
    }

    #[test]
    fn test_organize_by_date_preserves_filename() -> io::Result<()> {
        let source_dir = tempdir()?;
        let dest_dir = tempdir()?;

        let mut source_file = NamedTempFile::new_in(source_dir.path())?;
        source_file.write_all(b"Test")?;
        source_file.flush()?;

        let source_filename = source_file.path().file_name().unwrap().to_str().unwrap();

        let date = NaiveDate::from_ymd_opt(2023, 7, 4).unwrap();
        let result = organize_by_date(source_file.path(), dest_dir.path(), date)?;

        let dest_filename = result.file_name().unwrap().to_str().unwrap();
        assert_eq!(source_filename, dest_filename);

        Ok(())
    }

    #[test]
    fn test_organize_by_date_special_location_names() -> io::Result<()> {
        let source_dir = tempdir()?;
        let dest_dir = tempdir()?;

        let special_names = vec!["New York", "São Paulo", "Tokyo"];
        let date = NaiveDate::from_ymd_opt(2023, 8, 15).unwrap();

        for name in special_names {
            let mut source_file = NamedTempFile::new_in(source_dir.path())?;
            source_file.write_all(b"Test")?;
            source_file.flush()?;

            let result = organize_by_date_and_location(
                source_file.path(),
                dest_dir.path(),
                date,
                name,
            )?;

            assert!(result.to_string_lossy().contains(name));
        }

        Ok(())
    }
}

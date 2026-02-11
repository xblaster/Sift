use chrono::NaiveDate;
use std::fs;
use std::path::{Path, PathBuf};
use std::io;

/// Organize a file into a chronological folder structure (YYYY/MM/DD)
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

/// Organize a file into a folder structure with geographic cluster subfolder
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
    fn test_organize_by_date() -> io::Result<()> {
        let source_dir = tempdir()?;
        let dest_dir = tempdir()?;

        let mut source_file = NamedTempFile::new_in(source_dir.path())?;
        source_file.write_all(b"Test image")?;
        source_file.flush()?;

        let date = NaiveDate::from_ymd_opt(2023, 10, 15).unwrap();
        let result = organize_by_date(source_file.path(), dest_dir.path(), date)?;

        assert!(result.exists());
        assert!(result.to_string_lossy().contains("2023/10/15"));

        Ok(())
    }

    #[test]
    fn test_organize_by_date_and_location() -> io::Result<()> {
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
}

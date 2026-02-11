use chrono::{DateTime, Local, NaiveDate};
use exif::Reader;
use std::fs;
use std::io;
use std::path::Path;
use std::time::SystemTime;

#[derive(Debug, Clone)]
pub struct PhotoMetadata {
    pub file_path: String,
    pub date_taken: NaiveDate,
}

/// Extract EXIF date from a file with fallback to mtime
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

/// Extract date from file, logging errors gracefully
pub fn extract_date_safe<P: AsRef<Path>>(path: P) -> Option<NaiveDate> {
    extract_date(path).ok()
}

/// Build chronological folder path (YYYY/MM/DD)
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
    fn test_extract_date_from_mtime() -> io::Result<()> {
        let mut temp_file = NamedTempFile::new()?;
        temp_file.write_all(b"Test")?;
        temp_file.flush()?;

        let date = extract_date(temp_file.path())?;
        assert!(date <= Local::now().naive_local().date());
        Ok(())
    }
}

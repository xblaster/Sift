//! Orchestration module for the organize command.
//!
//! This module handles the high-level coordination of the photo organization pipeline,
//! including index loading, file discovery, analysis, clustering, and file operations.

use std::fs;
use std::io;
use std::path::PathBuf;
use chrono::NaiveDate;
use rayon::prelude::*;

use crate::hash;
use crate::index::Index;
use crate::metadata;
use crate::organization;

/// Context for an organize operation.
///
/// Holds all configuration and state needed for a photo organization run.
/// This struct encapsulates the settings that control how photos are discovered,
/// analyzed, organized, and indexed.
///
/// # Fields
///
/// * `source` - Source directory containing photos to organize
/// * `destination` - Destination directory for organized photos
/// * `with_clustering` - Whether to enable geographic clustering (optional)
/// * `jobs` - Number of parallel workers (None = auto-detect CPU count)
/// * `index_path` - Path to load/save index file (None = use default `.sift_index.bin`)
///
/// # Examples
///
/// ```no_run
/// # use std::path::PathBuf;
/// # use sift::organize::OrganizeContext;
/// let ctx = OrganizeContext::new(
///     PathBuf::from("/photos/source"),
///     PathBuf::from("/photos/organized"),
///     false,
///     Some(4),
///     None,
/// );
/// ```
#[derive(Debug, Clone)]
pub struct OrganizeContext {
    /// Source directory containing photos to organize
    pub source: PathBuf,
    /// Destination directory for organized photos
    pub destination: PathBuf,
    /// Whether to enable geographic clustering
    pub with_clustering: bool,
    /// Number of parallel workers (None = auto-detect CPU count)
    pub jobs: Option<usize>,
    /// Path to load/save index file (None = use default)
    pub index_path: Option<PathBuf>,
}

impl OrganizeContext {
    /// Creates a new OrganizeContext with the given parameters.
    ///
    /// # Arguments
    ///
    /// * `source` - Source directory path containing photos
    /// * `destination` - Destination directory path for organized photos
    /// * `with_clustering` - Enable geographic clustering
    /// * `jobs` - Number of parallel workers (None for auto-detect)
    /// * `index_path` - Custom index path (None for default `.sift_index.bin`)
    ///
    /// # Returns
    ///
    /// A new OrganizeContext instance configured with the given parameters.
    pub fn new(
        source: PathBuf,
        destination: PathBuf,
        with_clustering: bool,
        jobs: Option<usize>,
        index_path: Option<PathBuf>,
    ) -> Self {
        OrganizeContext {
            source,
            destination,
            with_clustering,
            jobs,
            index_path,
        }
    }

    /// Gets the path to the index file, using the default if not specified.
    ///
    /// If a custom index path was provided during construction, returns that path.
    /// Otherwise, returns the default path: `{destination}/.sift_index.bin`
    ///
    /// # Returns
    ///
    /// The path to the index file to use for this organization operation.
    pub fn get_index_path(&self) -> PathBuf {
        self.index_path.clone().unwrap_or_else(|| {
            self.destination.join(".sift_index.bin")
        })
    }
}

/// Represents a file record after analysis.
///
/// Contains metadata about a photo file that has been analyzed for hashing,
/// date extraction, and geographic information. This record is used throughout
/// the organization pipeline to track file attributes.
///
/// # Fields
///
/// * `path` - Original path to the file
/// * `hash` - Blake3 hash of the file contents (hex string)
/// * `date` - Extracted date from file metadata (for chronological organization)
/// * `location` - GPS coordinates (latitude, longitude) if available (for clustering)
#[derive(Debug, Clone)]
pub struct FileRecord {
    /// Original file path
    pub path: PathBuf,
    /// Blake3 hash of the file
    pub hash: String,
    /// Extracted date from metadata
    pub date: Option<NaiveDate>,
    /// GPS coordinates if available (lat, lon)
    pub location: Option<(f64, f64)>,
}

/// Statistics for an organize operation.
///
/// Tracks metrics about the organization process, including counts of files
/// at each stage (scanned, analyzed, organized, duplicates, failures).
/// This allows users to understand the results and impact of the organization run.
///
/// # Fields
///
/// * `files_scanned` - Total unique files discovered in source
/// * `files_analyzed` - Files successfully hashed and analyzed
/// * `files_skipped_duplicates` - Files skipped because already in index
/// * `files_organized` - Files successfully copied to destination
/// * `files_failed` - Files that encountered errors during organization
#[derive(Debug, Default, Clone)]
pub struct OrganizeStats {
    /// Total files discovered
    pub files_scanned: usize,
    /// Files successfully hashed and analyzed
    pub files_analyzed: usize,
    /// Files skipped as duplicates
    pub files_skipped_duplicates: usize,
    /// Files successfully organized
    pub files_organized: usize,
    /// Files that failed
    pub files_failed: usize,
}

/// Main orchestrator for photo organization.
///
/// Coordinates all stages of the photo organization pipeline:
/// 1. Index loading
/// 2. Source directory scanning
/// 3. File analysis (hashing, metadata extraction)
/// 4. Deduplication against existing index
/// 5. File organization and copying
/// 6. Index persistence
///
/// The orchestrator manages the overall flow and error handling,
/// while delegating specific operations to specialized modules.
pub struct Orchestrator {
    context: OrganizeContext,
    stats: OrganizeStats,
    errors: Vec<String>,
}

impl Orchestrator {
    /// Creates a new Orchestrator with the given context.
    ///
    /// # Arguments
    ///
    /// * `context` - Configuration and settings for the organize operation
    ///
    /// # Returns
    ///
    /// A new Orchestrator instance ready to coordinate a photo organization run.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use std::path::PathBuf;
    /// # use sift::organize::{OrganizeContext, Orchestrator};
    /// let ctx = OrganizeContext::new(
    ///     PathBuf::from("/source"),
    ///     PathBuf::from("/dest"),
    ///     false,
    ///     None,
    ///     None,
    /// );
    /// let orchestrator = Orchestrator::new(ctx);
    /// // Can now call orchestrator.run()
    /// ```
    pub fn new(context: OrganizeContext) -> Self {
        Orchestrator {
            context,
            stats: OrganizeStats::default(),
            errors: Vec::new(),
        }
    }

    /// Runs the complete organize pipeline.
    ///
    /// Stages:
    /// 1. Load index from destination
    /// 2. Scan source directory for photo files
    /// 3. Analyze files: hash and extract metadata
    /// 4. Deduplicate against index
    /// 5. Optionally cluster by location
    /// 6. Organize into destination folder structure
    /// 7. Save updated index
    pub fn run(&mut self) -> io::Result<OrganizeStats> {
        eprintln!("Starting photo organization...");
        eprintln!("Source: {:?}", self.context.source);
        eprintln!("Destination: {:?}", self.context.destination);

        // Stage 1: Load index
        eprintln!("Loading index...");
        let mut index = self.load_index()?;
        eprintln!("Index loaded: {} entries", index.len());

        // Stage 2: Scan source
        eprintln!("Scanning source directory...");
        let files = self.scan_source()?;
        self.stats.files_scanned = files.len();
        eprintln!("Found {} files", files.len());

        if files.is_empty() {
            eprintln!("No files to process");
            return Ok(self.stats.clone());
        }

        // Stage 3: Analyze files
        eprintln!("Analyzing files...");
        let records = self.analyze_files(&files)?;
        self.stats.files_analyzed = records.len();
        eprintln!("Analyzed {} files", records.len());

        // Stage 4: Deduplicate
        eprintln!("Deduplicating...");
        let unique_records: Vec<_> = records
            .into_iter()
            .filter(|record| {
                if index.contains_hash(&record.hash) {
                    eprintln!("Skipping duplicate: {:?}", record.path);
                    self.stats.files_skipped_duplicates += 1;
                    false
                } else {
                    true
                }
            })
            .collect();

        eprintln!(
            "After dedup: {} unique files",
            unique_records.len()
        );

        // Stage 5: Organize files
        eprintln!("Organizing files...");
        for record in unique_records {
            match self.organize_file(&record) {
                Ok(_) => {
                    self.stats.files_organized += 1;
                    // Add to index
                    index.add_entry(record.hash, record.path.to_string_lossy().to_string());
                }
                Err(e) => {
                    let err_msg = format!("Failed to organize {:?}: {}", record.path, e);
                    eprintln!("{}", err_msg);
                    self.errors.push(err_msg);
                    self.stats.files_failed += 1;
                }
            }
        }

        // Stage 6: Save index
        eprintln!("Saving index...");
        let index_path = self.context.get_index_path();
        index.save_to_file(&index_path)?;
        eprintln!("Index saved to {:?}", index_path);

        eprintln!("\nOrganization complete!");
        eprintln!("Files organized: {}", self.stats.files_organized);
        eprintln!("Duplicates skipped: {}", self.stats.files_skipped_duplicates);
        eprintln!("Failed: {}", self.stats.files_failed);

        if !self.errors.is_empty() {
            eprintln!("\nErrors encountered:");
            for err in &self.errors {
                eprintln!("  - {}", err);
            }
        }

        Ok(self.stats.clone())
    }

    /// Loads the index from the destination directory.
    fn load_index(&self) -> io::Result<Index> {
        let index_path = self.context.get_index_path();
        if index_path.exists() {
            Index::load_from_file(&index_path)
        } else {
            Ok(Index::new())
        }
    }

    /// Scans the source directory for photo files.
    fn scan_source(&self) -> io::Result<Vec<PathBuf>> {
        let mut files = Vec::new();
        let photo_extensions = vec!["jpg", "jpeg", "png", "tiff", "raw", "heic"];

        for entry in fs::read_dir(&self.context.source)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() {
                if let Some(ext) = path.extension() {
                    let ext_lower = ext.to_string_lossy().to_lowercase();
                    if photo_extensions.contains(&ext_lower.as_str()) {
                        files.push(path);
                    }
                }
            }
        }

        Ok(files)
    }

    /// Analyzes files: computes hashes and extracts metadata.
    fn analyze_files(&self, files: &[PathBuf]) -> io::Result<Vec<FileRecord>> {
        let records: Vec<FileRecord> = files
            .par_iter()
            .filter_map(|path| {
                match hash::hash_file(path) {
                    Ok(blake3_hash) => {
                        let hash_str = blake3_hash.to_hex().to_string();
                        let date = metadata::extract_date_safe(path);

                        Some(FileRecord {
                            path: path.clone(),
                            hash: hash_str,
                            date,
                            location: None, // TODO: Extract from EXIF GPS
                        })
                    }
                    Err(e) => {
                        eprintln!("Failed to hash {:?}: {}", path, e);
                        None
                    }
                }
            })
            .collect();

        Ok(records)
    }

    /// Organizes a single file to its destination.
    fn organize_file(&self, record: &FileRecord) -> io::Result<PathBuf> {
        let date = record.date.ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                "Cannot organize file without date",
            )
        })?;

        organization::organize_by_date(&record.path, &self.context.destination, date)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_organize_context_creation() {
        let ctx = OrganizeContext::new(
            PathBuf::from("/source"),
            PathBuf::from("/dest"),
            false,
            Some(4),
            None,
        );

        assert_eq!(ctx.source, PathBuf::from("/source"));
        assert_eq!(ctx.destination, PathBuf::from("/dest"));
        assert!(!ctx.with_clustering);
        assert_eq!(ctx.jobs, Some(4));
    }

    #[test]
    fn test_organize_context_default_index_path() {
        let ctx = OrganizeContext::new(
            PathBuf::from("/source"),
            PathBuf::from("/dest"),
            false,
            None,
            None,
        );

        let index_path = ctx.get_index_path();
        assert!(index_path.ends_with(".sift_index.bin"));
    }

    #[test]
    fn test_organize_context_custom_index_path() {
        let custom_path = PathBuf::from("/custom/index.bin");
        let ctx = OrganizeContext::new(
            PathBuf::from("/source"),
            PathBuf::from("/dest"),
            false,
            None,
            Some(custom_path.clone()),
        );

        let index_path = ctx.get_index_path();
        assert_eq!(index_path, custom_path);
    }

    #[test]
    fn test_stats_default() {
        let stats = OrganizeStats::default();
        assert_eq!(stats.files_scanned, 0);
        assert_eq!(stats.files_analyzed, 0);
        assert_eq!(stats.files_organized, 0);
    }

    #[test]
    fn test_file_record_creation() {
        let record = FileRecord {
            path: PathBuf::from("/source/photo.jpg"),
            hash: "abc123def456".to_string(),
            date: None,
            location: None,
        };

        assert_eq!(record.path, PathBuf::from("/source/photo.jpg"));
        assert_eq!(record.hash, "abc123def456");
        assert!(record.date.is_none());
        assert!(record.location.is_none());
    }

    #[test]
    fn test_file_record_with_date() {
        use chrono::NaiveDate;

        let date = NaiveDate::from_ymd_opt(2024, 2, 11);
        let record = FileRecord {
            path: PathBuf::from("/source/photo.jpg"),
            hash: "abc123".to_string(),
            date,
            location: None,
        };

        assert!(record.date.is_some());
        assert_eq!(record.date.unwrap().year(), 2024);
    }

    #[test]
    fn test_file_record_with_location() {
        let record = FileRecord {
            path: PathBuf::from("/source/photo.jpg"),
            hash: "abc123".to_string(),
            date: None,
            location: Some((37.7749, -122.4194)), // San Francisco
        };

        assert!(record.location.is_some());
        let (lat, lon) = record.location.unwrap();
        assert_eq!(lat, 37.7749);
        assert_eq!(lon, -122.4194);
    }

    #[test]
    fn test_scan_source_empty_directory() -> io::Result<()> {
        let temp = TempDir::new()?;
        let dest = TempDir::new()?;

        let ctx = OrganizeContext::new(
            temp.path().to_path_buf(),
            dest.path().to_path_buf(),
            false,
            None,
            None,
        );

        let orchestrator = Orchestrator::new(ctx);
        let files = orchestrator.scan_source()?;

        assert_eq!(files.len(), 0);
        Ok(())
    }

    #[test]
    fn test_scan_source_with_photos() -> io::Result<()> {
        let temp = TempDir::new()?;
        let dest = TempDir::new()?;

        // Create test photo files
        fs::write(temp.path().join("photo1.jpg"), "test")?;
        fs::write(temp.path().join("photo2.jpeg"), "test")?;
        fs::write(temp.path().join("photo3.png"), "test")?;
        fs::write(temp.path().join("document.txt"), "test")?; // Should be ignored

        let ctx = OrganizeContext::new(
            temp.path().to_path_buf(),
            dest.path().to_path_buf(),
            false,
            None,
            None,
        );

        let orchestrator = Orchestrator::new(ctx);
        let files = orchestrator.scan_source()?;

        assert_eq!(files.len(), 3, "Should find 3 photo files (not txt)");
        Ok(())
    }

    #[test]
    fn test_orchestrator_new() {
        let ctx = OrganizeContext::new(
            PathBuf::from("/source"),
            PathBuf::from("/dest"),
            false,
            None,
            None,
        );

        let orchestrator = Orchestrator::new(ctx.clone());

        assert_eq!(orchestrator.stats.files_scanned, 0);
        assert_eq!(orchestrator.stats.files_analyzed, 0);
        assert_eq!(orchestrator.errors.len(), 0);
    }

    #[test]
    fn test_organize_context_clone() {
        let ctx = OrganizeContext::new(
            PathBuf::from("/source"),
            PathBuf::from("/dest"),
            true,
            Some(8),
            Some(PathBuf::from("/custom/index.bin")),
        );

        let cloned = ctx.clone();

        assert_eq!(ctx.source, cloned.source);
        assert_eq!(ctx.destination, cloned.destination);
        assert_eq!(ctx.with_clustering, cloned.with_clustering);
        assert_eq!(ctx.jobs, cloned.jobs);
        assert_eq!(ctx.index_path, cloned.index_path);
    }

    #[test]
    fn test_stats_with_values() {
        let mut stats = OrganizeStats::default();
        stats.files_scanned = 100;
        stats.files_analyzed = 95;
        stats.files_skipped_duplicates = 5;
        stats.files_organized = 90;
        stats.files_failed = 0;

        assert_eq!(stats.files_scanned, 100);
        assert_eq!(stats.files_organized, 90);
        assert_eq!(stats.files_skipped_duplicates, 5);
    }

    #[test]
    fn test_stats_clone() {
        let stats = OrganizeStats {
            files_scanned: 50,
            files_analyzed: 48,
            files_skipped_duplicates: 2,
            files_organized: 46,
            files_failed: 2,
        };

        let cloned = stats.clone();
        assert_eq!(stats.files_scanned, cloned.files_scanned);
        assert_eq!(stats.files_organized, cloned.files_organized);
    }
}

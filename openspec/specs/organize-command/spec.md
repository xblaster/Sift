# Capability: Organize Command

## Purpose
TBD - The primary command for organizing photos from source to destination with deduplication, chronological sorting, and optional geographic clustering.

## Requirements

### Requirement: Command invocation and argument parsing
The system SHALL accept an `organize` command with source and destination directory arguments, and optional flags for clustering and job count configuration.

#### Scenario: Invoke organize with required arguments
- **WHEN** user executes `sift organize /source/path /dest/path`
- **THEN** the system accepts the command and begins processing

#### Scenario: Invoke organize with clustering flag
- **WHEN** user executes `sift organize /source /dest --with-clustering`
- **THEN** the system enables geographic clustering in the pipeline

#### Scenario: Invoke organize with job count
- **WHEN** user executes `sift organize /source /dest --jobs 8`
- **THEN** the system configures Rayon with the specified thread pool size

### Requirement: Index initialization and loading
The system SHALL load the deduplication index from the destination directory at startup, creating an empty index if none exists.

#### Scenario: Load existing index
- **WHEN** organize command starts and index file exists at destination
- **THEN** the system deserializes the index and loads all previously-seen file hashes into memory

#### Scenario: Create index on first run
- **WHEN** organize command starts and no index file exists
- **THEN** the system creates an empty HashMap and proceeds with processing

### Requirement: Multi-threaded file discovery
The system SHALL recursively discover all photo files in the source directory using single-threaded sequential scanning.

#### Scenario: Discover files recursively
- **WHEN** organize command processes source directory
- **THEN** the system enumerates all files in source and subdirectories

#### Scenario: Filter by recognized photo format
- **WHEN** a file with extension `.jpg`, `.jpeg`, `.png`, `.tiff`, `.raw`, or `.heic` is discovered
- **THEN** the file is queued for analysis

### Requirement: Parallel metadata analysis
The system SHALL analyze files in parallel using Rayon, computing Blake3 hashes and extracting EXIF metadata for each file.

#### Scenario: Hash computed in parallel
- **WHEN** multiple files are queued for analysis
- **THEN** the system uses Rayon `par_iter()` to compute hashes on available CPU cores

#### Scenario: EXIF extraction on photos
- **WHEN** a photo file is analyzed
- **THEN** the system extracts EXIF metadata (DateTimeOriginal, CreateDate, GPS coordinates)

#### Scenario: Handle files without EXIF
- **WHEN** a photo file lacks EXIF DateTimeOriginal/CreateDate
- **THEN** the system falls back to extracting mtime or parsing filename for date

### Requirement: Deduplication check
The system SHALL check each file's hash against the loaded index and skip processing for duplicate hashes.

#### Scenario: Skip duplicate file
- **WHEN** a file is analyzed and its hash already exists in the index
- **THEN** the system logs the duplicate and skips all subsequent processing stages for that file

#### Scenario: Process new file
- **WHEN** a file is analyzed and its hash does not exist in the index
- **THEN** the system proceeds to clustering and organization stages

### Requirement: Geographic clustering (when enabled)
The system SHALL apply DBSCAN geographic clustering to group photos by location when the `--with-clustering` flag is set.

#### Scenario: Cluster photos by location
- **WHEN** clustering is enabled and photos contain GPS coordinates
- **THEN** the system groups nearby photos using DBSCAN (ε ≈ 1km) and reverse-geocodes location names

#### Scenario: Organize without clustering
- **WHEN** clustering is disabled
- **THEN** the system organizes photos into chronological folders only (YYYY/MM/DD/)

### Requirement: Chronological organization
The system SHALL organize all files into a YYYY/MM/DD/ folder hierarchy based on the extracted date.

#### Scenario: Organize by extracted date
- **WHEN** a file has an extracted date of 2024-02-15
- **THEN** the file is assigned target path `{dest}/2024/02/15/{filename}`

#### Scenario: Organize with location subfolder
- **WHEN** clustering is enabled and file is grouped into location "San Francisco"
- **THEN** the file is assigned target path `{dest}/2024/02/15/San_Francisco/{filename}`

### Requirement: Atomic file operations
The system SHALL copy or move files to the destination with error handling and retry logic for network I/O failures.

#### Scenario: Copy file to organized destination
- **WHEN** a file is ready for organization
- **THEN** the system copies the file from source to target destination path, creating parent directories as needed

#### Scenario: Handle network I/O failure with retry
- **WHEN** a copy operation fails due to network timeout or temporary error
- **THEN** the system applies exponential backoff and retries up to N times before logging an error

#### Scenario: Skip file on persistent error
- **WHEN** a file copy fails permanently after max retries
- **THEN** the system logs the error, records it in a failure report, and continues processing other files

### Requirement: Index persistence
The system SHALL atomically update and persist the index at completion, recording all newly-processed files.

#### Scenario: Update index after successful processing
- **WHEN** organize command completes successfully
- **THEN** the system writes a new index file containing all previously-seen hashes plus hashes from this run

#### Scenario: Atomic index swap
- **WHEN** writing the updated index
- **THEN** the system writes to a temporary file and renames it atomically to prevent corruption on failure

### Requirement: Idempotence guarantee
The system SHALL ensure that running organize multiple times on the same source produces identical results (same folder structure, no re-processing of files).

#### Scenario: Re-run organize on same source
- **WHEN** organize is executed twice on the same source and destination
- **THEN** the second run detects all files from the first run in the index and skips them, producing identical output

#### Scenario: Organize new files after first run
- **WHEN** organize is run a second time after new files are added to source
- **THEN** the system processes only the new files and maintains the existing folder structure for previously-processed files

### Requirement: Error handling and logging
The system SHALL log meaningful messages for all stages and gracefully handle errors without aborting on file-level failures.

#### Scenario: Log file processing progress
- **WHEN** organize is running with verbose output
- **THEN** the system prints progress messages (files scanned, hashed, copied, skipped)

#### Scenario: Log duplicate detection
- **WHEN** a duplicate file is detected
- **THEN** the system logs the original location and the duplicate file path

#### Scenario: Log critical errors and continue
- **WHEN** an unrecoverable error occurs (e.g., destination full, index corruption)
- **THEN** the system logs the error with context and continues processing other files if possible, or terminates gracefully with error message

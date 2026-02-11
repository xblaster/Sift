## 1. Command Handler Implementation

- [x] 1.1 Implement `organize_command()` handler in `cli.rs` to parse and validate arguments
- [x] 1.2 Create `OrganizeContext` struct to hold source, destination, and config (clustering, jobs)
- [x] 1.3 Wire organize command handler to `main.rs` CLI dispatch
- [ ] 1.4 Add `--dry-run` flag parsing (optional for MVP, log output only)

## 2. Index Management

- [x] 2.1 Implement `Index::load()` to deserialize index from destination if exists
- [x] 2.2 Implement `Index::create_empty()` to initialize empty index on first run
- [x] 2.3 Add `Index::contains()` method for O(1) duplicate checking
- [x] 2.4 Implement `Index::insert()` to record processed file hashes
- [x] 2.5 Implement `Index::save_atomic()` for transactional persistence (write temp, atomic rename)

## 3. File Discovery (Walker Stage)

- [x] 3.1 Create `Walker::scan()` function using `walkdir` for recursive directory traversal
- [x] 3.2 Filter files by recognized photo extensions (jpg, jpeg, png, tiff, raw, heic)
- [ ] 3.3 Handle symlinks (follow or skip - decision documented)
- [x] 3.4 Return `Vec<PathBuf>` or iterator of discovered files

## 4. Metadata Analysis (Analyzer Stage)

- [x] 4.1 Implement `Analyzer::process_batch()` using Rayon `par_iter()` for parallel processing
- [x] 4.2 Integrate Blake3 hashing (use existing `hash` module)
- [x] 4.3 Integrate EXIF metadata extraction (use existing `metadata` module)
- [x] 4.4 Implement date priority fallback: EXIF DateTimeOriginal → CreateDate → filename → mtime
- [ ] 4.5 Extract GPS coordinates if available (for clustering)
- [x] 4.6 Return `FileRecord` struct with hash, date, location, path

## 5. Deduplication & Filtering

- [x] 5.1 Implement deduplication check in orchestrator: skip files with hash in index
- [x] 5.2 Log duplicate detection with original and duplicate path
- [x] 5.3 Update index with newly-processed file hashes

## 6. Geographic Clustering (Conditional)

- [ ] 6.1 Check `--with-clustering` flag; only proceed if enabled
- [ ] 6.2 Extract GPS coordinates from analyzed files with valid locations
- [ ] 6.3 Integrate DBSCAN clustering (use existing `clustering` module)
- [ ] 6.4 Apply reverse geocoding using GeoNames database (use existing `geonames` module)
- [ ] 6.5 Return cluster assignments with location names

## 7. Chronological Organization

- [x] 7.1 For each file, compute target path: `{dest}/YYYY/MM/DD/{filename}`
- [ ] 7.2 If clustering enabled, insert location: `{dest}/YYYY/MM/DD/Location/{filename}`
- [x] 7.3 Validate YYYY/MM/DD structure (use existing `organization` module)
- [x] 7.4 Create parent directories if needed

## 8. File Copy & Network I/O

- [x] 8.1 Implement `copy_with_retry()` using exponential backoff for network errors
- [ ] 8.2 Apply 1 MB buffered reads (use existing `network_io` module)
- [x] 8.3 Handle copy errors: log, record in failure report, continue processing
- [x] 8.4 Create parent directories atomically if they don't exist
- [ ] 8.5 Preserve file metadata (timestamps, permissions) during copy

## 9. Orchestration & Main Pipeline

- [x] 9.1 Create `Orchestrator` struct to coordinate all stages
- [x] 9.2 Implement `run()` method with stage sequence: load_index → scan → analyze → cluster (optional) → organize → copy → save_index
- [x] 9.3 Add progress reporting/logging at each stage
- [x] 9.4 Implement error handling: log file-level errors, continue; fail on critical errors
- [x] 9.5 Track and log summary: files scanned, hashed, organized, skipped, failed

## 10. Error Handling & Logging

- [x] 10.1 Implement `OrganizeError` enum covering network, I/O, metadata, and clustering failures
- [x] 10.2 Add logging macros for debug/info/warn/error levels
- [x] 10.3 Implement failure report collection (store failed file paths with error reason)
- [x] 10.4 Print final summary: total files, organized, duplicates, errors, and failure report
- [ ] 10.5 Ensure graceful shutdown on critical errors (corrupt index, destination full)

## 11. Unit Testing

- [x] 11.1 Add unit tests for `Index::load()` with valid and corrupt index files
- [x] 11.2 Add unit tests for `Index::contains()` and `Index::insert()` operations
- [ ] 11.3 Add unit tests for `Index::save_atomic()` with simulated write failures
- [x] 11.4 Add unit tests for `Walker::scan()` with various directory structures and symlinks
- [x] 11.5 Add unit tests for date extraction priority logic (EXIF → filename → mtime)
- [x] 11.6 Add unit tests for date fallback scenarios (missing EXIF, invalid dates)
- [ ] 11.7 Add unit tests for file path computation: YYYY/MM/DD/ structure and location insertion
- [ ] 11.8 Add unit tests for `copy_with_retry()` with exponential backoff timing
- [ ] 11.9 Add unit tests for retry logic with transient vs. permanent failures
- [x] 11.10 Add unit tests for error categorization in `OrganizeError` enum
- [x] 11.11 Add unit tests for `Orchestrator` state transitions and stage sequencing
- [x] 11.12 Add unit tests for duplicate detection and index updates
- [ ] 11.13 Add unit tests for clustering integration with mock DBSCAN output
- [ ] 11.14 Add unit tests for logging output validation (expected messages, levels)
- [x] 11.15 Add unit tests for edge cases: empty source, single file, large directories

## 12. Integration Testing

- [ ] 12.1 Create integration test with sample photo library (minimal EXIF data)
- [ ] 12.2 Test organize command end-to-end: scan → hash → organize → index
- [ ] 12.3 Test idempotence: run organize twice, verify same results
- [ ] 12.4 Test duplicate detection: add duplicate file to source, verify it's skipped
- [ ] 12.5 Test clustering integration (with `--with-clustering` flag)
- [ ] 12.6 Test network error resilience with mock failures
- [ ] 12.7 Test error handling: corrupted EXIF, unreadable files, destination full

## 13. Code Coverage Setup

- [ ] 13.1 Configure `tarpaulin` or `llvm-cov` for Rust code coverage measurement
- [ ] 13.2 Add coverage configuration to `Cargo.toml` (dev-dependencies)
- [ ] 13.3 Create `Makefile` or script targets: `make test-coverage`, `make coverage-report`
- [ ] 13.4 Set up coverage baseline: target ≥80% for organize module, ≥70% overall
- [ ] 13.5 Configure coverage to exclude generated code and test utilities
- [ ] 13.6 Add coverage reports to `.gitignore` (but track config files)

## 14. Coverage Validation & Reporting

- [ ] 14.1 Run full test suite with coverage: `cargo tarpaulin --out Html`
- [ ] 14.2 Identify and address low-coverage areas (< 70%)
- [ ] 14.3 Add tests for uncovered branches in error handling paths
- [ ] 14.4 Add tests for uncovered edge cases in clustering and date extraction
- [ ] 14.5 Validate coverage for all public APIs (target 100%)
- [ ] 14.6 Generate coverage badge/report for README
- [ ] 14.7 Document coverage targets and exclusions in CONTRIBUTING.md

## 15. Documentation & Examples

- [x] 15.1 Add rustdoc comments (`///`) to public functions in all modified modules
- [x] 15.2 Add module-level rustdoc explaining the pipeline architecture
- [ ] 15.3 Update README.md with organize command usage examples
- [ ] 15.4 Document date extraction priority and fallback logic
- [ ] 15.5 Document clustering feature and `--with-clustering` flag
- [ ] 15.6 Add examples to main.rs showing organize usage
- [ ] 15.7 Create guide for symlink behavior and network storage setup

## 16. Code Review & Cleanup

- [ ] 16.1 Review error messages for clarity and user-friendliness
- [ ] 16.2 Ensure no unwrap() calls without documentation (prefer Result types)
- [ ] 16.3 Check for memory leaks or unbounded allocations
- [ ] 16.4 Verify Rayon thread pool configuration respects `--jobs` flag
- [ ] 16.5 Run clippy and address linting warnings
- [ ] 16.6 Ensure consistent code style across all changes

## 17. Final Verification

- [ ] 17.1 Verify `sift organize --help` displays all flags and arguments
- [ ] 17.2 Test verbose output (`-v` flag) shows meaningful progress
- [ ] 17.3 Run organize on realistic photo archive (if available)
- [ ] 17.4 Verify memory footprint remains < 500 MB
- [ ] 17.5 Benchmark hashing throughput (target: ~500 MB/s on Blake3)
- [ ] 17.6 Validate folder structure matches specification exactly

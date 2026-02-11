## Why

The `organize` command is currently a stub in the CLI that prints "Organization feature not yet implemented". The full photo organization pipeline—the core value of Sift—needs implementation to deliver the end-to-end workflow: scanning source files, analyzing metadata, clustering by date/geography, and writing organized results to the destination. This is the primary user-facing feature.

## What Changes

- Implement the complete `organize` command handler integrating all four pipeline stages (Walker → Analyzer → Clusterer → Writer)
- Load/initialize the deduplication index from destination before processing
- Execute multi-threaded directory scanning of source
- Analyze files in parallel: Blake3 hashing + EXIF metadata extraction
- Cluster photos by chronological date (YYYY/MM/DD/) and geographic location
- Copy/move organized files to destination with proper folder hierarchy
- Write updated index atomically for idempotence
- Add comprehensive documentation: inline rustdoc comments, architectural overview, and CLI usage examples

## Capabilities

### New Capabilities
- `organize-command`: Full implementation of the `organize` CLI command that orchestrates the complete photo organization pipeline from source to destination.

### Modified Capabilities
- `core-deduplication`: Use the existing hash index to track processed files and prevent re-processing
- `chronological-organization`: Apply date extraction and folder structure creation for organized output
- `geographic-clustering`: Apply DBSCAN clustering to group photos by location
- `network-io-optimization`: Apply buffered I/O and retry logic during copy operations

## Impact

- **Code**: Additions/modifications to `cli.rs` (command handler), new coordination logic in `main.rs`, integration with existing modules (`hash`, `metadata`, `clustering`, `organization`, `network_io`)
- **APIs**: The `organize` command becomes fully functional with proper error handling and logging
- **Documentation**: Add rustdoc to public functions, update README with organize examples, create module-level architecture guide
- **Testing**: Integration tests for end-to-end workflow; unit tests for pipeline stage coordination
- **Breaking Changes**: None - this implements existing planned functionality

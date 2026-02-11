## Context

**Current State:**
- The `organize` CLI command exists as a stub in `cli.rs` that only prints placeholder messages
- Four architectural modules are in place but not orchestrated: `hash`, `metadata`, `clustering`, `organization`, `network_io`
- An index system is designed for deduplication but not yet integrated into the organize workflow
- The ARCHITECTURE.md defines the 4-stage pipeline (Walker → Analyzer → Clusterer → Writer) but implementation is incomplete

**Constraints:**
- Must maintain idempotence: running organize multiple times on the same source should produce identical results
- Must handle network I/O resilience (SMB/NFS) with exponential backoff, not fail fast
- Memory footprint must remain constant regardless of source size (< 500 MB for 1M+ photos)
- Must support multi-threaded processing (Rayon) without blocking network reads

**Stakeholders:**
- End users: photographers, sysadmins, photo studios organizing large archives
- Maintainers: need clear separation of concerns between pipeline stages

## Goals / Non-Goals

**Goals:**
- Implement a fully functional `organize` command that orchestrates all four pipeline stages
- Achieve idempotence through index-based deduplication and atomic index updates
- Process photos in parallel (Rayon) for hashing and EXIF extraction
- Organize output into chronological (YYYY/MM/DD) and geographic folder hierarchies
- Apply network I/O optimizations (buffered reads, exponential backoff)
- Include comprehensive documentation: rustdoc, integration tests, CLI examples
- Ensure zero-duplicate guarantees via Blake3 hashing

**Non-Goals:**
- Implement real-time directory watching or incremental sync (out of scope for this change)
- Add GUI or TUI interface (CLI only)
- Support custom folder naming schemes (fixed hierarchy defined in proposal)
- Optimize for single-threaded or single-file use cases (focus on batch operations)
- Add configuration file support beyond CLI flags

## Decisions

### 1. Pipeline Orchestration Pattern: Staged Stream Processing
**Choice:** Implement a staged, non-buffered pipeline where each stage processes a batch of files sequentially, passing results to the next stage.

**Rationale:**
- Avoids creating large in-memory buffers of processed files
- Aligns with the "constant memory footprint" requirement
- Simpler error handling and backpressure management than async channels

**Alternative Considered:** Channel-based async pipeline (Tokio). Rejected because Rust network I/O on SMB/NFS benefits more from simple blocking reads with retry logic than from full async scheduling overhead.

### 2. Index Loading & Updates: Load-Modify-Atomic-Write
**Choice:**
- Load entire index into a `HashMap<String, FileRecord>` at startup (keyed by file hash)
- Check all files against this in-memory map (O(1) dedup)
- Write updated index atomically at end via rename to avoid corruption on failure

**Rationale:**
- Maintains strict idempotence: index serves as "seen" set
- Network optimization: no round-trip I/O for each file check
- Simplicity: HashMap operations are trivial; atomic rename is OS-level guarantee

**Alternative Considered:** Streaming checks against remote database. Rejected due to latency impact on network storage.

### 3. Concurrency Strategy: Rayon for Analysis, Sequential I/O for Walking
**Choice:**
- Walker stage: Use `walkdir` sequentially to discover files (metadata only)
- Analyzer stage: Use Rayon `par_iter()` to hash and extract EXIF in parallel
- Clusterer & Writer stages: Sequential (clustering is CPU-light; writing is I/O-serialized anyway)

**Rationale:**
- Parallel hashing maximizes throughput on multi-core systems
- Sequential walking avoids thundering herd problem on shared network storage
- Analyzer parallelism is the biggest bottleneck; later stages are I/O or CPU-trivial

**Alternative Considered:** Full async pipeline with Tokio. Rejected for the same reason as decision #1.

### 4. Error Handling: Log and Continue vs. Fail Fast
**Choice:** For file-level errors (unreadable file, bad EXIF), log and skip the file; continue processing others. For fatal errors (index corruption, destination full), fail immediately with clear error message.

**Rationale:**
- Batch operations in archives often encounter one or two corrupted files
- Strict idempotence requires not halting on recoverable errors
- Users can re-run organize on failed source files later

**Alternative Considered:** Stop on first error. Rejected because it's incompatible with idempotence and real-world photo archives.

### 5. Index Serialization: Bincode (Existing)
**Choice:** Continue using the existing `bincode` crate for index serialization (as documented in ARCHITECTURE.md).

**Rationale:**
- Already chosen in initial project for speed and minimal overhead
- Binary format ensures index is not human-editable (prevents corruption)

**Alternative Considered:** JSON (human-readable). Rejected because it's slower and doesn't address safety concerns.

### 6. Documentation Strategy: Inline Rustdoc + Integration Tests + README Examples
**Choice:**
- Add rustdoc comments (`///`) to all public functions in modules
- Create integration test (`tests/organize_integration.rs`) demonstrating end-to-end workflow
- Update README with organize examples showing typical usage patterns
- Add module-level architecture guide in comments referencing ARCHITECTURE.md

**Rationale:**
- Rustdoc keeps documentation close to code, reduces drift
- Integration tests serve as executable documentation
- README examples help users understand feature capabilities

## Risks / Trade-offs

| Risk | Impact | Mitigation |
|------|--------|-----------|
| **Network timeouts during copy phase** | User-visible delay or failure | Implement exponential backoff in network_io module; log retry attempts |
| **Disk space exhaustion on destination** | Partial organization, index corruption | Check free space before writing; fail fast if insufficient |
| **Index corruption on power loss** | Idempotence broken; duplicate files on next run | Use atomic rename to swap old/new index; document recovery procedure |
| **Parallel hashing contention on SMB** | Network bottleneck if source is slow | Accept this trade-off; hashing is still faster than serial; document expected throughput |
| **EXIF extraction failure on corrupted images** | Fallback to filename/mtime date extraction | Implement graceful fallback; log warnings for corrupted files |

## Migration Plan

1. **Phase 1**: Implement `organize` command handler as orchestrator; wire it to existing module interfaces
2. **Phase 2**: Integrate index loading/saving into the organize workflow
3. **Phase 3**: Add error handling, logging, and retry logic
4. **Phase 4**: Write integration tests and documentation
5. **Deployment**: No breaking changes; existing CLI flags should work as designed

## Open Questions

- Should we add a `--dry-run` flag to preview organization without copying? (Deferred to post-MVP)
- Should we support custom date priority rules via CLI flags? (Deferred; ARCHITECTURE.md priority is sufficient)
- How should we handle symlinks in source? (Assume: follow them; document explicitly)

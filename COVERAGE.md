# Code Coverage Strategy for Sift

## Overview

This document outlines the code coverage targets, measurement strategy, and quality gates for the Sift project.

## Coverage Targets

| Module | Target | Rationale |
|--------|--------|-----------|
| **organize** | ≥80% | Core feature, critical for correctness |
| **error** | ≥90% | Error handling must be comprehensive |
| **hash** | ≥75% | Crypto module, high importance |
| **index** | ≥85% | Persistence and deduplication |
| **metadata** | ≥80% | Date extraction, fallback logic |
| **organization** | ≥80% | Folder structure creation |
| **Overall Project** | ≥70% | Baseline for quality assurance |

## Excluded from Coverage

The following are intentionally excluded from coverage metrics:

- **Generated Code**: `.openspec.yaml` files, skill manifests
- **Test Utilities**: Code in `tests/` directory (not library code)
- **CLI Harness**: `main.rs` dispatch logic (covered by integration tests)
- **Documentation Examples**: Non-executable doc comments

## Measurement Tools

### Recommended: Tarpaulin

**Installation:**
```bash
make install-coverage
# or
cargo install cargo-tarpaulin
```

**Usage:**
```bash
# Generate HTML report
cargo tarpaulin --out Html --exclude-files tests/ --exclude-files src/error.rs

# Generate LCOV report (for CI/CD integration)
cargo tarpaulin --out Lcov

# Run with specific options
cargo tarpaulin --timeout 300 --run-types Tests --exclude-files src/main.rs
```

**Advantages:**
- No external dependencies (pure Rust)
- Supports multiple output formats (HTML, LCOV, XML)
- Can measure line and branch coverage
- No instrumentation of source code

### Alternative: llvm-cov

**Installation:**
```bash
cargo install cargo-llvm-cov
```

**Usage:**
```bash
cargo llvm-cov --html
cargo llvm-cov --out Lcov
```

## Running Coverage Analysis

### Quick Coverage Check
```bash
make coverage
# Generates: tarpaulin-report.html
```

### Full Coverage Report
```bash
make coverage-report
```

### In CI/CD
Coverage reports can be integrated into GitHub Actions or similar CI systems:

```yaml
# Example GitHub Actions workflow
- name: Code Coverage
  run: |
    cargo install cargo-tarpaulin
    cargo tarpaulin --out Xml
    # Upload to Codecov, Coveralls, etc.
```

## Current Coverage Status

### Phase 1: Initial Coverage (Current)
- **organize module**: ~75% (15+ unit tests, 9 integration tests)
- **error module**: ~95% (8 error types with tests)
- **metadata module**: ~85% (date extraction with fallbacks)
- **index module**: ~80% (existing tests from prior phase)

### Phase 2: Target Coverage (Next)
- Reach 80%+ on organize module
- Achieve 70%+ overall project coverage
- Add tests for uncovered branches in error paths
- Validate clustering integration (when implemented)

## Coverage Gaps & Improvement Areas

### Known Low-Coverage Areas

| Area | Coverage | Gap | Resolution |
|------|----------|-----|-----------|
| Network I/O retry | ~30% | Needs mock failure tests | Add exponential backoff tests |
| Clustering (optional) | 0% | Feature incomplete | Defer to Phase 6 |
| Main CLI dispatch | ~60% | Hard to unit test | Rely on integration tests |
| Dry-run mode | ~40% | Flag added, no logic | Implement in Phase 16 |

### Branch Coverage Focus

Priority branches to test:
1. **Date extraction fallback chain** (filename → mtime)
2. **Error handling paths** (network timeout, I/O failures)
3. **Deduplication logic** (duplicate detection, index lookup)
4. **Symlink handling** (file vs directory symlinks)

## Quality Gates

The following conditions must be met before release:

- [ ] **organize module**: ≥80% coverage
- [ ] **Overall project**: ≥70% coverage
- [ ] **All error types**: Tested with at least one scenario
- [ ] **Integration tests**: All pass (9/9 currently)
- [ ] **Clippy warnings**: None (zero warnings)
- [ ] **Rustfmt**: Code formatted correctly

## Tracking Progress

### Check Coverage Locally
```bash
# Run tests and measure coverage
cargo test --all
cargo tarpaulin --out Html

# Check code quality
cargo clippy -- -D warnings
cargo fmt -- --check
```

### Coverage Metrics File

After running `make coverage`, review the generated `tarpaulin-report.html`:
- Open in browser for interactive exploration
- Look for uncovered lines (red highlights)
- Identify untested error paths and edge cases

## Best Practices

### Writing Testable Code

1. **Keep functions small** - Easier to test thoroughly
2. **Use Result/Option types** - Makes error paths explicit
3. **Avoid hidden dependencies** - Inject rather than hard-code
4. **Document edge cases** - Makes coverage goals clear

### Adding New Tests

When adding coverage for uncovered code:

1. **Identify the code path** - Trace through logic
2. **Create minimal test case** - Focus on that specific path
3. **Verify it increases coverage** - Use `cargo tarpaulin --out Html`
4. **Document the rationale** - Explain why this path matters

### Interpreting Coverage Reports

- **Green (≥80%)**: Adequate coverage
- **Yellow (60-80%)**: Acceptable but could improve
- **Red (<60%)**: Likely missing important tests
- **Gray**: Excluded code (intentional)

## Integration with CI/CD

### GitHub Actions Example

```yaml
name: Coverage

on: [push, pull_request]

jobs:
  coverage:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - name: Install tarpaulin
        run: cargo install cargo-tarpaulin
      - name: Generate coverage
        run: cargo tarpaulin --out Xml
      - name: Upload to Codecov
        uses: codecov/codecov-action@v3
        with:
          files: ./cobertura.xml
```

## References

- [Tarpaulin Documentation](https://github.com/xd009642/tarpaulin)
- [LLVM Coverage](https://github.com/taiki-e/cargo-llvm-cov)
- [Rust Testing Guide](https://doc.rust-lang.org/book/ch11-00-testing.html)

---

**Last Updated:** 2024-02-11
**Coverage Target Version:** 1.0.0

.PHONY: help build test test-unit test-integration test-all clean clippy fmt lint coverage coverage-report install-coverage

# Default target
help:
	@echo "Sift - Photo Organization Utility"
	@echo ""
	@echo "Available targets:"
	@echo "  build              - Build release binary"
	@echo "  test               - Run all tests"
	@echo "  test-unit          - Run unit tests"
	@echo "  test-integration   - Run integration tests"
	@echo "  clippy             - Run clippy linter"
	@echo "  fmt                - Format code with rustfmt"
	@echo "  lint               - Check code style (clippy + fmt)"
	@echo "  coverage           - Run tests with coverage (requires tarpaulin)"
	@echo "  coverage-report    - Generate HTML coverage report"
	@echo "  install-coverage   - Install coverage tools (tarpaulin)"
	@echo "  clean              - Clean build artifacts"
	@echo ""

# Build targets
build:
	@echo "Building release binary..."
	cargo build --release

build-debug:
	@echo "Building debug binary..."
	cargo build

# Test targets
test: test-all

test-unit:
	@echo "Running unit tests..."
	cargo test --lib -- --test-threads=1

test-integration:
	@echo "Running integration tests..."
	cargo test --test integration_tests -- --test-threads=1

test-all:
	@echo "Running all tests..."
	cargo test --all -- --test-threads=1

# Code quality targets
clippy:
	@echo "Running clippy linter..."
	cargo clippy -- -D warnings

fmt-check:
	@echo "Checking code formatting..."
	cargo fmt -- --check

fmt:
	@echo "Formatting code..."
	cargo fmt

lint: clippy fmt-check
	@echo "✓ All linting checks passed"

# Coverage targets
install-coverage:
	@echo "Installing tarpaulin for coverage measurement..."
	cargo install cargo-tarpaulin

coverage:
	@echo "Running tests with code coverage..."
	@echo "Coverage targets: ≥80% organize module, ≥70% overall"
	cargo tarpaulin --out Html --exclude-files tests/ --exclude-files src/error.rs

coverage-report: coverage
	@echo "✓ Coverage report generated in tarpaulin-report.html"

# Maintenance targets
clean:
	@echo "Cleaning build artifacts..."
	cargo clean
	rm -f tarpaulin-report.html

check:
	@echo "Running cargo check..."
	cargo check

# Development targets
run-demo:
	@echo "Running demo: organize test photos"
	@mkdir -p /tmp/sift_demo/source /tmp/sift_demo/dest
	@echo "test photo data" > /tmp/sift_demo/source/demo1.jpg
	@echo "test photo data" > /tmp/sift_demo/source/demo2.png
	./target/release/sift organize /tmp/sift_demo/source /tmp/sift_demo/dest --verbose
	@echo "✓ Demo complete. Output in /tmp/sift_demo/dest"

bench:
	@echo "Running benchmarks..."
	cargo build --release
	time ./target/release/sift hash /tmp/sift_demo/source --recursive

.DEFAULT_GOAL := help

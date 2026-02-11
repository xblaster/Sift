#!/bin/bash
# Test runner script for Sift
# Usage: ./test.sh [unit|integration|all]

set -e

TEST_TYPE="${1:-all}"

case "$TEST_TYPE" in
    unit)
        echo "Running unit tests..."
        cargo test --lib -- --test-threads=1
        ;;
    integration)
        echo "Running integration tests..."
        cargo test --test integration_tests -- --test-threads=1
        ;;
    all)
        echo "Running all tests..."
        cargo test --all -- --test-threads=1
        ;;
    *)
        echo "Usage: ./test.sh [unit|integration|all]"
        exit 1
        ;;
esac

echo "✓ All tests passed"

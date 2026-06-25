#!/bin/bash
# Pre-push CI check — run the same checks GitHub Actions will run.
# Usage: ./scripts/ci-check.sh

set -e
cd "$(dirname "$0")/.."

echo "=== cargo test ==="
cargo test -- --nocapture

echo "=== cargo clippy ==="
cargo clippy

echo "=== cargo fmt --check ==="
cargo fmt --check

echo "=== all good ==="

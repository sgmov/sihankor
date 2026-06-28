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

echo "=== SiHankor self-governance (--warn) ==="
# --warn 模式：报告全部问题但不阻断
# 在 fix 完 53 个 F 违规后可以改为 --strict
cargo run --quiet --bin rebuild_index -- --warn

echo "=== all good ==="

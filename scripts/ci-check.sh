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

echo "=== SiHankor self-governance (--strict) ==="
# --strict 模式：F 级违规 + parse/db 错误即阻断
# 53 F 违规已清账（plan 260628-2030-ci-self-govern），现在 CI 是真正门禁
# 新规则（upstream 链完整性、stage 转换合法性）默认不阻断，等数据积累后单独 plan 升级
cargo run --quiet --bin rebuild_index -- --strict

echo "=== all good ==="

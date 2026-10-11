#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

echo "== Chunk 13: object capability checkpoint =="

echo
echo "== formatting =="
cargo fmt --all --check

echo
echo "== workspace tests =="
cargo test --workspace

echo
echo "== workspace clippy =="
cargo clippy \
  --workspace \
  --all-targets \
  -- \
  -D warnings

echo
echo "== object storage end-to-end =="
./scripts/check-objects-e2e.sh "chunk-13-checkpoint"

echo
echo "== diff hygiene =="
git diff --check

echo
echo "Chunk 13 checkpoint passed."

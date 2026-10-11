#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

echo "== Chunk 14: Environment Specification checkpoint =="

echo
echo "== formatting =="
cargo fmt --all --check

echo
echo "== environment specification tests =="
cargo test -p presolve-environment

echo
echo "== environment-driven resolver proof =="
cargo test \
  -p presolve-resolver \
  --test environment_spec_resolution

echo
echo "== environment-driven node materialization proof =="
cargo test \
  -p presolve-node \
  --test environment_materialization

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
echo "== diff hygiene =="
git diff --check

echo
echo "Chunk 14 checkpoint passed."

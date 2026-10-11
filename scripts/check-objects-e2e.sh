#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

TARGET="wasm32-wasip2"
MANIFEST="examples/objects-demo/Cargo.toml"
COMPONENT="examples/objects-demo/target/${TARGET}/debug/presolve_example_objects_demo.wasm"
INPUT="${1:-presolve-e2e}"

if ! rustup target list --installed | grep -qx "$TARGET"; then
  echo "missing Rust target: $TARGET" >&2
  echo "install it with: rustup target add $TARGET" >&2
  exit 1
fi

cargo build \
  --manifest-path "$MANIFEST" \
  --target "$TARGET"

cargo run \
  -p presolve-node \
  --example objects_e2e \
  -- \
  "$COMPONENT" \
  "$INPUT"

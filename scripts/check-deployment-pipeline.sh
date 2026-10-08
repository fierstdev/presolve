#!/usr/bin/env bash

set -euo pipefail

readonly MANIFEST="examples/kv-demo/Cargo.toml"
readonly CONTRACT="examples/kv-demo/presolve.toml"
readonly TARGET="wasm32-wasip2"
readonly COMPONENT="examples/kv-demo/target/${TARGET}/debug/presolve_example_kv_demo.wasm"

echo "== build key/value component =="

cargo build \
  --manifest-path "${MANIFEST}" \
  --target "${TARGET}"

echo
echo "== validate key/value component =="

wasm-tools validate "${COMPONENT}"

echo
echo "== resolve and materialize deployment plan =="

cargo run \
  --quiet \
  -p presolve-node \
  --example deployment \
  -- "${CONTRACT}" "${COMPONENT}"

echo
echo "Presolve deployment pipeline check passed"

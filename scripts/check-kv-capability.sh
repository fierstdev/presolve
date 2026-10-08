#!/usr/bin/env bash

set -euo pipefail

readonly MANIFEST="examples/kv-demo/Cargo.toml"
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
echo "== verify capability import =="

COMPONENT_WIT="$(
  wasm-tools component wit "${COMPONENT}"
)"

if ! grep -q \
  "presolve:kv/store@0.1.0" \
  <<< "${COMPONENT_WIT}"
then
  echo "failure: expected key/value capability import"
  echo
  echo "${COMPONENT_WIT}"
  exit 1
fi

echo "✓ component imports presolve:kv/store@0.1.0"

echo
echo "== execute provider substitution check =="

cargo run \
  --quiet \
  -p presolve-runtime \
  --example capability \
  -- "${COMPONENT}"

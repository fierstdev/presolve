#!/usr/bin/env bash

set -euo pipefail

readonly MANIFEST="examples/hello/Cargo.toml"
readonly TARGET="wasm32-wasip2"
readonly COMPONENT="examples/hello/target/${TARGET}/debug/presolve_example_hello.wasm"

echo "== build hello component =="

cargo build \
  --manifest-path "${MANIFEST}" \
  --target "${TARGET}"

echo
echo "== validate component =="

wasm-tools validate "${COMPONENT}"

echo
echo "== component interface =="

wasm-tools component wit "${COMPONENT}"

echo
echo "hello component check passed"
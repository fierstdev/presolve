#!/usr/bin/env bash

set -euo pipefail

readonly TARGET="wasm32-wasip2"

readonly HELLO_MANIFEST="examples/hello/Cargo.toml"
readonly HELLO_COMPONENT="examples/hello/target/${TARGET}/debug/presolve_example_hello.wasm"

readonly ADVERSARIAL_MANIFEST="examples/adversarial/Cargo.toml"
readonly ADVERSARIAL_COMPONENT="examples/adversarial/target/${TARGET}/debug/presolve_example_adversarial.wasm"

echo "== build hello component =="

cargo build \
  --manifest-path "${HELLO_MANIFEST}" \
  --target "${TARGET}"

echo
echo "== build adversarial component =="

cargo build \
  --manifest-path "${ADVERSARIAL_MANIFEST}" \
  --target "${TARGET}"

echo
echo "== validate components =="

wasm-tools validate "${HELLO_COMPONENT}"
wasm-tools validate "${ADVERSARIAL_COMPONENT}"

echo
echo "== verify wall-clock deadlines and isolation =="

cargo run \
  --quiet \
  -p presolve-runtime \
  --example deadlines \
  -- \
  "${HELLO_COMPONENT}" \
  "${ADVERSARIAL_COMPONENT}"

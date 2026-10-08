#!/usr/bin/env bash

set -euo pipefail

readonly MANIFEST="examples/hello/Cargo.toml"
readonly TARGET="wasm32-wasip2"
readonly COMPONENT="examples/hello/target/${TARGET}/debug/presolve_example_hello.wasm"
readonly INPUT="Austin"
readonly EXPECTED="Hello from Presolve, Austin."

echo "== build hello component =="

cargo build \
  --manifest-path "${MANIFEST}" \
  --target "${TARGET}"

echo
echo "== validate hello component =="

wasm-tools validate "${COMPONENT}"

echo
echo "== execute through Presolve runtime =="

OUTPUT="$(
  cargo run \
    --quiet \
    -p presolve-runtime \
    --example invoke \
    -- "${COMPONENT}" "${INPUT}"
)"

echo "${OUTPUT}"

if [[ "${OUTPUT}" != "${EXPECTED}" ]]; then
  echo
  echo "runtime output mismatch"
  echo "expected: ${EXPECTED}"
  echo "actual:   ${OUTPUT}"
  exit 1
fi

echo
echo "Presolve runtime check passed"

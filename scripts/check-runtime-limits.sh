#!/usr/bin/env bash

set -euo pipefail

readonly MANIFEST="examples/adversarial/Cargo.toml"
readonly TARGET="wasm32-wasip2"
readonly COMPONENT="examples/adversarial/target/${TARGET}/debug/edgezero_example_adversarial.wasm"

echo "== build adversarial component =="

cargo build \
  --manifest-path "${MANIFEST}" \
  --target "${TARGET}"

echo
echo "== validate adversarial component =="

wasm-tools validate "${COMPONENT}"

echo
echo "== verify infinite execution is terminated =="

set +e

SPIN_OUTPUT="$(
  cargo run \
    --quiet \
    -p edgezero-runtime \
    --example invoke \
    -- "${COMPONENT}" spin 2>&1
)"

SPIN_STATUS=$?

set -e

if [[ "${SPIN_STATUS}" -eq 0 ]]; then
  echo "failure: infinite component completed successfully"
  exit 1
fi

if ! grep -qi "fuel" <<< "${SPIN_OUTPUT}"; then
  echo "failure: infinite component failed for an unexpected reason"
  echo
  echo "${SPIN_OUTPUT}"
  exit 1
fi

echo "✓ infinite execution exhausted its fuel budget"

echo
echo "== verify excessive memory growth is rejected =="

set +e

MEMORY_OUTPUT="$(
  cargo run \
    --quiet \
    -p edgezero-runtime \
    --example invoke \
    -- "${COMPONENT}" memory 2>&1
)"

MEMORY_STATUS=$?

set -e

if [[ "${MEMORY_STATUS}" -eq 0 ]]; then
  echo "failure: excessive memory allocation completed successfully"
  echo
  echo "${MEMORY_OUTPUT}"
  exit 1
fi

echo "✓ excessive memory growth was rejected"

echo
echo "EdgeZero runtime limit checks passed"

#!/usr/bin/env bash

set -euo pipefail

readonly MANIFEST="examples/kv-demo/Cargo.toml"
readonly CONTRACT="examples/kv-demo/presolve.toml"
readonly TARGET="wasm32-wasip2"
readonly COMPONENT="examples/kv-demo/target/${TARGET}/debug/presolve_example_kv_demo.wasm"

readonly OUTPUT_A="${TMPDIR:-/tmp}/presolve-kv-demo-a.presolve"
readonly OUTPUT_B="${TMPDIR:-/tmp}/presolve-kv-demo-b.presolve"

cleanup() {
  rm -f "${OUTPUT_A}" "${OUTPUT_B}"
}

trap cleanup EXIT

echo "== build component =="

cargo build \
  --manifest-path "${MANIFEST}" \
  --target "${TARGET}"

echo
echo "== validate component =="

wasm-tools validate "${COMPONENT}"

echo
echo "== package first release =="

cargo run \
  --quiet \
  -p presolve-bundle \
  --example package \
  -- "${CONTRACT}" "${COMPONENT}" "${OUTPUT_A}"

echo
echo "== package second release =="

cargo run \
  --quiet \
  -p presolve-bundle \
  --example package \
  -- "${CONTRACT}" "${COMPONENT}" "${OUTPUT_B}"

echo
echo "== verify deterministic bundle bytes =="

if ! cmp -s "${OUTPUT_A}" "${OUTPUT_B}"; then
  echo "failure: identical inputs produced different .presolve bundles"
  exit 1
fi

echo "✓ identical inputs produced identical bundle bytes"

echo
echo "== verify bundle contents =="

ENTRIES="$(tar -tf "${OUTPUT_A}")"

if ! grep -qx "manifest.json" <<< "${ENTRIES}"; then
  echo "failure: bundle is missing manifest.json"
  exit 1
fi

if ! grep -qx "artifacts/main.wasm" <<< "${ENTRIES}"; then
  echo "failure: bundle is missing artifacts/main.wasm"
  exit 1
fi

ENTRY_COUNT="$(printf '%s\n' "${ENTRIES}" | wc -l | tr -d ' ')"

if [[ "${ENTRY_COUNT}" != "2" ]]; then
  echo "failure: expected 2 bundle entries, found ${ENTRY_COUNT}"
  echo "${ENTRIES}"
  exit 1
fi

echo "✓ bundle contains canonical manifest and component artifact"

echo
echo "== inspect manifest for deployment leakage =="

MANIFEST_JSON="$(tar -xOf "${OUTPUT_A}" manifest.json)"

for forbidden in \
  '"environment_id"' \
  '"provider_id"' \
  '"deployment_plan"'
do
  if grep -q "${forbidden}" <<< "${MANIFEST_JSON}"; then
    echo "failure: bundle manifest contains deployment-specific field ${forbidden}"
    exit 1
  fi
done

echo "✓ bundle contains no deployment-specific identity"

echo
echo "Presolve bundle check passed"

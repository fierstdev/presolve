#!/usr/bin/env bash

set -euo pipefail

readonly MANIFEST="examples/multi-workload/Cargo.toml"
readonly CONTRACT="examples/multi-workload/presolve.toml"
readonly TARGET="wasm32-wasip2"

readonly API="examples/multi-workload/target/${TARGET}/debug/presolve_example_multi_api.wasm"
readonly WORKER="examples/multi-workload/target/${TARGET}/debug/presolve_example_multi_worker.wasm"

readonly OUTPUT_A="${TMPDIR:-/tmp}/presolve-multi-a.presolve"
readonly OUTPUT_B="${TMPDIR:-/tmp}/presolve-multi-b.presolve"

cleanup() {
  rm -f "${OUTPUT_A}" "${OUTPUT_B}"
}

trap cleanup EXIT

echo "== build multi-workload application =="

cargo build \
  --manifest-path "${MANIFEST}" \
  --target "${TARGET}"

echo
echo "== validate components =="

wasm-tools validate "${API}"
wasm-tools validate "${WORKER}"

echo "✓ api component is valid"
echo "✓ worker component is valid"

echo
echo "== package api then worker =="

cargo run \
  --quiet \
  -p presolve-bundle \
  --example package-components \
  -- \
  "${CONTRACT}" \
  "${OUTPUT_A}" \
  "api=${API}" \
  "worker=${WORKER}"

echo
echo "== package worker then api =="

cargo run \
  --quiet \
  -p presolve-bundle \
  --example package-components \
  -- \
  "${CONTRACT}" \
  "${OUTPUT_B}" \
  "worker=${WORKER}" \
  "api=${API}"

echo
echo "== verify deterministic application release =="

if ! cmp -s "${OUTPUT_A}" "${OUTPUT_B}"; then
  echo "failure: component input order changed bundle bytes"
  exit 1
fi

echo "✓ component input order does not affect bundle bytes"

echo
echo "== verify canonical bundle contents =="

ENTRIES="$(tar -tf "${OUTPUT_A}")"

EXPECTED=$'manifest.json\nartifacts/api.wasm\nartifacts/worker.wasm'

if [[ "${ENTRIES}" != "${EXPECTED}" ]]; then
  echo "failure: unexpected multi-workload bundle contents"
  echo
  echo "expected:"
  echo "${EXPECTED}"
  echo
  echo "found:"
  echo "${ENTRIES}"
  exit 1
fi

echo "✓ canonical multi-workload artifact layout"

echo
echo "== verify manifest workload composition =="

MANIFEST_JSON="$(tar -xOf "${OUTPUT_A}" manifest.json)"

python3 - "${MANIFEST_JSON}" <<'PY'
import json
import sys

manifest = json.loads(sys.argv[1])

workloads = manifest["workloads"]

assert [workload["name"] for workload in workloads] == [
    "api",
    "worker",
]

assert workloads[0]["kind"] == "component"
assert workloads[1]["kind"] == "component"

assert workloads[0]["artifacts"][0]["path"] == "artifacts/api.wasm"
assert workloads[1]["artifacts"][0]["path"] == "artifacts/worker.wasm"
PY

echo "✓ manifest contains canonical api and worker workloads"

echo
echo "Presolve multi-workload bundle check passed"

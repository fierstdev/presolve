#!/usr/bin/env bash
set -euo pipefail
example=examples/production-resume
out="$example/.presolve"
cleanup() { rm -rf "$out"; }
trap cleanup EXIT
test -s "$example/presolve.json"
test -s "$example/src/ComputedDiamond.tsx"
cargo run -q -p presolve-cli -- build "$example/src/ComputedDiamond.tsx" --out "$out" --production >/dev/null
test -s "$out/production.runtime.json"
test -s "$out/resume.runtime.json"
test -d "$out/production"
rg --quiet '"schemaVersion":1' "$out/production.runtime.json"
rg --quiet '"schema_version":6' "$out/resume.runtime.json"
RUST_TEST_THREADS=1 cargo test -q -p presolve-cli --test runtime_browser phase_k_production_artifact_runs_under_csp_and_rejects_malformed_boot_in_a_real_browser -- --nocapture
./scripts/verify-l14b-explicit-workspace-example.sh
git diff --check

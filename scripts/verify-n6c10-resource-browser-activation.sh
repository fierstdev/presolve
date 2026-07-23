#!/usr/bin/env bash
set -euo pipefail
repo_root="$(git rev-parse --show-toplevel)"
cd "$repo_root"
test -s docs/specifications/phase-n/PHASE_N_N6C10_RESOURCE_BROWSER_ACTIVATION_CONTRACT.md
cargo test -q -p presolve-compiler --lib runtime_codegen::tests::emits_runtime_manifest_bootstrap -- --exact
RUST_TEST_THREADS=1 cargo test -q -p presolve-cli --test runtime_browser host_bound_resource_endpoint_activates_in_a_real_browser -- --exact
cargo check -q -p presolve-compiler
cargo fmt --all --check
git diff --check

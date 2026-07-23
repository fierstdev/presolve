#!/usr/bin/env bash
set -euo pipefail
repo_root="$(git rev-parse --show-toplevel)"
cd "$repo_root"
test -s docs/specifications/phase-n/PHASE_N_N6C7_RESOURCE_ARTIFACT_MODULE_BINDING_CONTRACT.md
cargo test -q -p presolve-compiler --lib runtime_resource_artifact::tests::requires_exact_runtime_module_location_for_execution_facing_artifact -- --exact
cargo check -q -p presolve-compiler
cargo fmt --all --check
git diff --check

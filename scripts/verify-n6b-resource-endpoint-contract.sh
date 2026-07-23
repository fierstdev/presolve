#!/usr/bin/env bash
set -euo pipefail
repo_root="$(git rev-parse --show-toplevel)"; cd "$repo_root"
test -s docs/specifications/phase-n/PHASE_N_N6B_RESOURCE_ENDPOINT_CONTRACT.md
rg --fixed-strings --quiet 'resource_endpoint' docs/specifications/phase-n/PHASE_N_N6B_RESOURCE_ENDPOINT_CONTRACT.md
cargo test -q -p presolve-compiler --lib semantic_package::tests::resource_exports_require_a_closed_endpoint_contract
cargo test -q -p presolve-compiler registry_is_versioned_stable_and_explains_deferred_families --lib
cargo check -q -p presolve-compiler
cargo fmt --all --check
git diff --check

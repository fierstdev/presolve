#!/usr/bin/env bash
set -euo pipefail
repo_root="$(git rev-parse --show-toplevel)"; cd "$repo_root"
test -s docs/specifications/phase-n/PHASE_N_N6_RESOURCE_FOUNDATION_CONTRACT.md
rg --fixed-strings --quiet 'N6-A establishes the first executable compiler product' docs/specifications/phase-n/PHASE_N_N6_RESOURCE_FOUNDATION_CONTRACT.md
rg --fixed-strings --quiet 'ResourceDeclaration' crates/presolve_compiler/src/resource.rs
cargo test -q -p presolve-compiler --lib resource::tests
cargo test -q -p presolve-compiler registry_is_versioned_stable_and_explains_deferred_families --lib
cargo check -q -p presolve-compiler
cargo fmt --all --check
git diff --check

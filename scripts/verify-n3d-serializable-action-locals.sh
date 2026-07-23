#!/usr/bin/env bash
set -euo pipefail
repo_root="$(git rev-parse --show-toplevel)"
cd "$repo_root"
test -s docs/specifications/phase-n/PHASE_N_N3D_SERIALIZABLE_ACTION_LOCALS_CONTRACT.md
rg --fixed-strings --quiet 'emits the existing literal `assign` State operation' docs/specifications/phase-n/PHASE_N_N3D_SERIALIZABLE_ACTION_LOCALS_CONTRACT.md
cargo test -q -p presolve-compiler --lib serializable_action_local
cargo test -q -p presolve-compiler --lib registry_is_versioned_stable_and_explains_deferred_families
RUST_TEST_THREADS=1 cargo test -q -p presolve-cli --test runtime_browser serializable_action_local_updates_state_from_compiler_generated_runtime
cargo check -q -p presolve-compiler -p presolve-cli
cargo fmt --all --check
git diff --check

#!/usr/bin/env bash
set -euo pipefail
repo_root="$(git rev-parse --show-toplevel)"
cd "$repo_root"
test -s docs/specifications/phase-n/PHASE_N_N3A_SERIALIZABLE_STATE_REPLACEMENT_CONTRACT.md
rg --fixed-strings --quiet 'replace the complete State field' docs/specifications/phase-n/PHASE_N_N3A_SERIALIZABLE_STATE_REPLACEMENT_CONTRACT.md
cargo test -q -p presolve-compiler registry_is_versioned_stable_and_explains_deferred_families --lib
cargo test -q -p presolve-cli --test runtime_browser serializable_record_state_replacement_executes_from_compiler_generated_runtime
cargo check -q -p presolve-compiler -p presolve-cli
cargo fmt --all --check
git diff --check

#!/usr/bin/env bash
set -euo pipefail
repo_root="$(git rev-parse --show-toplevel)"
cd "$repo_root"
test -s docs/specifications/phase-n/PHASE_N_N3C_ACTION_PARAMETER_STATE_TYPE_CONTRACT.md
rg --fixed-strings --quiet 'rejected with `PSC1044`' docs/specifications/phase-n/PHASE_N_N3C_ACTION_PARAMETER_STATE_TYPE_CONTRACT.md
cargo test -q -p presolve-compiler --lib component_graph_rejects_action_parameter_state_type_mismatch
cargo test -q -p presolve-compiler --lib registry_is_versioned_stable_and_explains_deferred_families
cargo check -q -p presolve-compiler
cargo fmt --all --check
git diff --check

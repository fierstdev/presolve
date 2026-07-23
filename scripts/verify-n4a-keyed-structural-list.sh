#!/usr/bin/env bash
set -euo pipefail
repo_root="$(git rev-parse --show-toplevel)"; cd "$repo_root"
test -s docs/specifications/phase-n/PHASE_N_N4A_KEYED_STRUCTURAL_LIST_CONTRACT.md
rg --fixed-strings --quiet 'DOM position is' docs/specifications/phase-n/PHASE_N_N4A_KEYED_STRUCTURAL_LIST_CONTRACT.md
cargo test -q -p presolve-compiler registry_is_versioned_stable_and_explains_deferred_families --lib
cargo test -q -p presolve-cli --test runtime_browser keyed_lists_reconcile_in_a_real_browser
cargo fmt --all --check
git diff --check

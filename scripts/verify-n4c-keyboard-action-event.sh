#!/usr/bin/env bash
set -euo pipefail
repo_root="$(git rev-parse --show-toplevel)"
cd "$repo_root"
test -s docs/specifications/phase-n/PHASE_N_N4C_KEYBOARD_ACTION_EVENT_CONTRACT.md
rg --fixed-strings --quiet 'No keyboard payload' docs/specifications/phase-n/PHASE_N_N4C_KEYBOARD_ACTION_EVENT_CONTRACT.md
cargo test -q -p presolve-compiler --lib component_graph_reports_unsupported_event_errors
cargo test -q -p presolve-compiler --lib registry_is_versioned_stable_and_explains_deferred_families
RUST_TEST_THREADS=1 cargo test -q -p presolve-cli --test runtime_browser structured_serializable_action_local_executes_from_compiler_generated_runtime
cargo check -q -p presolve-compiler -p presolve-cli
cargo fmt --all --check
git diff --check

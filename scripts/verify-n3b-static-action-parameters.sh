#!/usr/bin/env bash
set -euo pipefail
repo_root="$(git rev-parse --show-toplevel)"
cd "$repo_root"
test -s docs/specifications/phase-n/PHASE_N_N3B_STATIC_ACTION_PARAMETERS_CONTRACT.md
rg --fixed-strings --quiet 'template manifest to schema version `5`' docs/specifications/phase-n/PHASE_N_N3B_STATIC_ACTION_PARAMETERS_CONTRACT.md
cargo test -q -p presolve-compiler --lib component_graph_validates_static_action_parameter_bindings
cargo test -q -p presolve-compiler --lib component_graph_rejects_unbound_action_parameters
cargo test -q -p presolve-compiler --lib component_graph_requires_action_decorator_for_parameter_state_assignment
cargo test -q -p presolve-compiler --lib registry_is_versioned_stable_and_explains_deferred_families
RUST_TEST_THREADS=1 cargo test -q -p presolve-cli --test runtime_browser static_callback_argument_updates_state_through_compiler_action_parameter
cargo check -q -p presolve-compiler -p presolve-cli
cargo fmt --all --check
git diff --check

#!/usr/bin/env bash
set -euo pipefail
repo_root="$(git rev-parse --show-toplevel)"
cd "$repo_root"
test -s docs/specifications/phase-n/PHASE_N_N5A_COMPONENT_INVOCATION_CONTRACT.md
rg --fixed-strings --quiet 'canonical invocation identity' docs/specifications/phase-n/PHASE_N_N5A_COMPONENT_INVOCATION_CONTRACT.md
cargo test -q -p presolve-cli --test component_fixtures component_composition_fixture_covers_topology_slots_caller_ownership_and_blocking
cargo test -q -p presolve-compiler --lib registry_is_versioned_stable_and_explains_deferred_families
cargo check -q -p presolve-compiler -p presolve-cli
cargo fmt --all --check
git diff --check

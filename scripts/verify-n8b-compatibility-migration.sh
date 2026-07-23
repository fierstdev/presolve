#!/usr/bin/env bash
set -euo pipefail

repo_root="$(git rev-parse --show-toplevel)"
cd "$repo_root"

readonly contract=docs/specifications/phase-n/PHASE_N_N8B_COMPATIBILITY_MIGRATION_CONTRACT.md
test -s "$contract"

cargo test -q -p presolve-compiler semantic_capability::tests::registry_is_versioned_stable_and_explains_deferred_families --lib
cargo test -q -p presolve-cli --test explain capability_registry_has_deterministic_json_human_and_migration_projections -- --exact
cargo run -q -p presolve-cli -- explain --capabilities --format migration | rg --fixed-strings --quiet 'Rejected syntax catalog'
cargo run -q -p presolve-cli -- explain --capabilities --format migration | rg --fixed-strings --quiet 'opaque_typescript | opaque | N9 must define opaque isolation'
cargo fmt --all --check
git diff --check

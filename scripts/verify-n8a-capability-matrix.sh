#!/usr/bin/env bash
set -euo pipefail

repo_root="$(git rev-parse --show-toplevel)"
cd "$repo_root"

readonly contract=docs/specifications/phase-n/PHASE_N_N8A_CAPABILITY_MATRIX_CONTRACT.md
test -s "$contract"

cargo test -q -p presolve-compiler semantic_capability::tests::registry_is_versioned_stable_and_explains_deferred_families --lib
cargo test -q -p presolve-cli --test explain capability_registry_has_deterministic_json_and_human_projections -- --exact
cargo run -q -p presolve-cli -- explain --capabilities --format json | rg --fixed-strings --quiet '"schema_version": 1'
cargo run -q -p presolve-cli -- explain --capabilities --format human | rg --fixed-strings --quiet 'opaque_typescript | opaque | deferred'
cargo fmt --all --check
git diff --check

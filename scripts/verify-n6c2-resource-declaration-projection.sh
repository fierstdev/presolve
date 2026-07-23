#!/usr/bin/env bash
set -euo pipefail

repo_root="$(git rev-parse --show-toplevel)"
cd "$repo_root"

test -s docs/specifications/phase-n/PHASE_N_N6C2_RESOURCE_DECLARATION_PROJECTION_CONTRACT.md
rg --fixed-strings --quiet 'PSC1046' docs/specifications/phase-n/PHASE_N_N6C2_RESOURCE_DECLARATION_PROJECTION_CONTRACT.md
cargo test -q -p presolve-compiler --lib tests::resolves_resource_source_designator_through_integrity_checked_package_contract -- --exact
cargo test -q -p presolve-compiler --lib semantic_type::tests::determines_structural_serialization_compatibility -- --exact
cargo test -q -p presolve-compiler --lib semantic_capability::tests::registry_is_versioned_stable_and_explains_deferred_families -- --exact
cargo check -q -p presolve-compiler
cargo fmt --all --check
git diff --check

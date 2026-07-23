#!/usr/bin/env bash
set -euo pipefail

repo_root="$(git rev-parse --show-toplevel)"
cd "$repo_root"

test -s docs/specifications/phase-n/PHASE_N_N6C1_RESOURCE_ENDPOINT_RESOLUTION_CONTRACT.md
rg --fixed-strings --quiet 'PSC1046' docs/specifications/phase-n/PHASE_N_N6C1_RESOURCE_ENDPOINT_RESOLUTION_CONTRACT.md
rg --fixed-strings --quiet 'N6-C1 is complete' docs/specifications/phase-n/PHASE_N_SEMANTIC_CAPABILITY_EXPANSION_PLAN.md
cargo test -q -p presolve-compiler --lib tests::resolves_resource_source_designator_through_integrity_checked_package_contract -- --exact
cargo test -q -p presolve-compiler --lib tests::component_graph_retains_non_executable_resource_declaration_facts -- --exact
cargo check -q -p presolve-compiler
cargo fmt --all --check
git diff --check

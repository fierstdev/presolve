#!/usr/bin/env bash
set -euo pipefail

repo_root="$(git rev-parse --show-toplevel)"
cd "$repo_root"

test -s docs/specifications/phase-n/PHASE_N_N6C5_RESOURCE_SOURCE_DIAGNOSTIC_CONTRACT.md
rg --fixed-strings --quiet 'PSC1128' docs/specifications/phase-n/PHASE_N_N6C5_RESOURCE_SOURCE_DIAGNOSTIC_CONTRACT.md
cargo test -q -p presolve-compiler --lib tests::resolves_resource_source_designator_through_integrity_checked_package_contract -- --exact
cargo test -q -p presolve-compiler --lib tests::component_graph_retains_non_executable_resource_declaration_facts -- --exact
cargo check -q -p presolve-compiler
cargo fmt --all --check
git diff --check

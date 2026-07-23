#!/usr/bin/env bash
set -euo pipefail

repo_root="$(git rev-parse --show-toplevel)"
cd "$repo_root"

test -s docs/specifications/phase-n/PHASE_N_N6C3_RESOURCE_ARTIFACT_CONTRACT.md
rg --fixed-strings --quiet 'PSC1046' docs/specifications/phase-n/PHASE_N_N6C3_RESOURCE_ARTIFACT_CONTRACT.md
cargo test -q -p presolve-compiler --lib runtime_resource_artifact::tests::projects_resolved_resource_declaration_and_idle_activation_deterministically -- --exact
cargo test -q -p presolve-compiler --lib tests::resolves_resource_source_designator_through_integrity_checked_package_contract -- --exact
cargo check -q -p presolve-compiler
cargo fmt --all --check
git diff --check

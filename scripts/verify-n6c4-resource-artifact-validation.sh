#!/usr/bin/env bash
set -euo pipefail

repo_root="$(git rev-parse --show-toplevel)"
cd "$repo_root"

test -s docs/specifications/phase-n/PHASE_N_N6C4_RESOURCE_ARTIFACT_VALIDATION_CONTRACT.md
rg --fixed-strings --quiet 'PSC1046' docs/specifications/phase-n/PHASE_N_N6C4_RESOURCE_ARTIFACT_VALIDATION_CONTRACT.md
cargo test -q -p presolve-compiler --lib runtime_resource_artifact::tests::projects_resolved_resource_declaration_and_idle_activation_deterministically -- --exact
cargo test -q -p presolve-compiler --lib runtime_resource_artifact::tests::rejects_malformed_resource_artifact_identity_and_lifecycle_records -- --exact
cargo check -q -p presolve-compiler
cargo fmt --all --check
git diff --check

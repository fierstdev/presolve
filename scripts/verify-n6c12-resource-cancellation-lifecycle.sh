#!/usr/bin/env bash
set -euo pipefail
repo_root="$(git rev-parse --show-toplevel)"
cd "$repo_root"
test -s docs/specifications/phase-n/PHASE_N_N6C12_RESOURCE_CANCELLATION_LIFECYCLE_CONTRACT.md
cargo test -q -p presolve-compiler --lib runtime_codegen::tests::emits_runtime_manifest_bootstrap -- --exact
cargo check -q -p presolve-compiler
cargo fmt --all --check
git diff --check

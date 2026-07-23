#!/usr/bin/env bash
set -euo pipefail
repo_root="$(git rev-parse --show-toplevel)"
cd "$repo_root"
test -s docs/specifications/phase-n/PHASE_N_N6C6_RESOURCE_RUNTIME_MODULE_ABI_CONTRACT.md
cargo test -q -p presolve-compiler --lib semantic_package_runtime::tests::resolves_only_the_exact_integrity_checked_runtime_module_coordinate -- --exact
cargo check -q -p presolve-compiler
cargo fmt --all --check
git diff --check

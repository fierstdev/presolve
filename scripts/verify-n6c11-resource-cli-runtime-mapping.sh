#!/usr/bin/env bash
set -euo pipefail
repo_root="$(git rev-parse --show-toplevel)"
cd "$repo_root"
test -s docs/specifications/phase-n/PHASE_N_N6C11_RESOURCE_CLI_RUNTIME_MAPPING_CONTRACT.md
rg --fixed-strings --quiet -- '--package-runtime' crates/presolve_cli/src/main.rs
cargo check -q -p presolve-compiler -p presolve-cli
cargo fmt --all --check
git diff --check

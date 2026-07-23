#!/usr/bin/env bash
set -euo pipefail
repo_root="$(git rev-parse --show-toplevel)"
cd "$repo_root"
test -s docs/specifications/phase-n/PHASE_N_N6C8_RESOURCE_PAGE_PUBLICATION_CONTRACT.md
rg --fixed-strings --quiet 'presolve-resources-runtime' crates/presolve_compiler/src/page_codegen.rs
cargo check -q -p presolve-compiler
cargo fmt --all --check
git diff --check

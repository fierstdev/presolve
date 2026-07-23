#!/usr/bin/env bash
set -euo pipefail
repo_root="$(git rev-parse --show-toplevel)"
cd "$repo_root"
test -s docs/specifications/phase-n/PHASE_N_N2D_OPTIONAL_MEMBER_ACCESS_CONTRACT.md
rg --fixed-strings --quiet '`8` by making optionality explicit' docs/specifications/phase-n/PHASE_N_N2D_OPTIONAL_MEMBER_ACCESS_CONTRACT.md
cargo test -q -p presolve-compiler emits_optional_member_read_with_compiler_retained_optionality --lib
cargo test -q -p presolve-cli --test runtime_browser optional_member_accesses_execute_from_compiler_generated_runtime_programs
cargo check -q -p presolve-parser -p presolve-compiler -p presolve-cli
cargo fmt --all --check
git diff --check

#!/usr/bin/env bash
set -euo pipefail
repo_root="$(git rev-parse --show-toplevel)"
cd "$repo_root"
test -s docs/specifications/phase-n/PHASE_N_N2E_BUILTIN_MATH_ABS_CONTRACT.md
rg --fixed-strings --quiet 'schema-v9' docs/specifications/phase-n/PHASE_N_N2E_BUILTIN_MATH_ABS_CONTRACT.md
cargo test -q -p presolve-compiler emits_compiler_registered_math_abs_as_unary_program --lib
cargo test -q -p presolve-compiler registry_is_versioned_stable_and_explains_deferred_families --lib
cargo test -q -p presolve-cli --test runtime_browser registered_math_abs_executes_from_compiler_generated_runtime_programs
cargo check -q -p presolve-compiler -p presolve-cli
cargo fmt --all --check
git diff --check

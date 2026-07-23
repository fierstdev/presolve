#!/usr/bin/env bash
set -euo pipefail
repo_root="$(git rev-parse --show-toplevel)"; cd "$repo_root"
test -s docs/specifications/phase-n/PHASE_N_N2F_BUILTIN_MATH_MIN_MAX_CONTRACT.md
rg --fixed-strings --quiet 'schema version `10`' docs/specifications/phase-n/PHASE_N_N2F_BUILTIN_MATH_MIN_MAX_CONTRACT.md
cargo test -q -p presolve-compiler --lib emits_compiler_registered_math_min_and_max_as_binary_programs
cargo test -q -p presolve-compiler registry_is_versioned_stable_and_explains_deferred_families --lib
RUST_TEST_THREADS=1 cargo test -q -p presolve-cli --test runtime_browser registered_math_min_max_execute_from_compiler_generated_runtime_programs
cargo check -q -p presolve-compiler -p presolve-cli
cargo fmt --all --check
git diff --check

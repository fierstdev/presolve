#!/usr/bin/env bash
set -euo pipefail
repo_root="$(git rev-parse --show-toplevel)"
cd "$repo_root"
test -s docs/specifications/phase-n/PHASE_N_N2G_BUILTIN_MATH_ROUNDING_CONTRACT.md
cargo test -q -p presolve-compiler --lib runtime_computed_artifact::tests::emits_compiler_registered_math_rounding_as_unary_programs -- --exact
cargo test -q -p presolve-compiler --lib semantic_capability::tests::registry_is_versioned_stable_and_explains_deferred_families -- --exact
RUST_TEST_THREADS=1 cargo test -q -p presolve-cli --test runtime_browser registered_math_rounding_executes_from_compiler_generated_runtime_programs -- --exact
cargo check -q -p presolve-compiler -p presolve-cli
cargo fmt --all --check
git diff --check

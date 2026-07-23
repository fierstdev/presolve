#!/usr/bin/env bash
set -euo pipefail
repo_root="$(git rev-parse --show-toplevel)"
cd "$repo_root"
readonly contract=docs/specifications/phase-n/PHASE_N_N2C_BOOLEAN_CONDITIONAL_CONTRACT.md
test -s "$contract"
for phrase in 'boolean or a boolean union' 'reports `PSC1029`' 'artifact schema `7`'; do
  rg --fixed-strings --quiet "$phrase" "$contract"
done
cargo test -q -p presolve-compiler emits_boolean_conditional_select_program --lib
cargo test -q -p presolve-compiler rejects_non_boolean_computed_conditional --lib
cargo test -q -p presolve-compiler registry_is_versioned_stable_and_explains_deferred_families --lib
cargo test -q -p presolve-cli --test runtime_browser boolean_conditional_computed_values_execute_from_compiler_generated_runtime_programs
cargo check -q -p presolve-compiler -p presolve-cli
cargo fmt --all --check
git diff --check

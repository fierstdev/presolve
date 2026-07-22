#!/usr/bin/env bash
set -euo pipefail

repo_root="$(git rev-parse --show-toplevel)"
cd "$repo_root"

readonly contract=docs/specifications/phase-n/PHASE_N_N2B_STATIC_INDEX_ACCESS_CONTRACT.md
test -s "$contract"
for phrase in \
  'string literal or a non-negative integer literal' \
  'own-property read' \
  'schema version `6`' \
  'does not execute authored source'; do
  rg --fixed-strings --quiet "$phrase" "$contract"
done

cargo test -q -p presolve-compiler \
  emits_static_index_access_program_for_tuple_state --lib
cargo test -q -p presolve-compiler \
  registry_is_versioned_stable_and_explains_deferred_families --lib
cargo test -q -p presolve-cli --test explain \
  build_command_writes_compiler_generated_computed_runtime_metadata
cargo test -q -p presolve-cli --test runtime_browser \
  static_index_accesses_execute_from_compiler_generated_runtime_programs
cargo check -q -p presolve-compiler -p presolve-cli
cargo fmt --all --check
git diff --check

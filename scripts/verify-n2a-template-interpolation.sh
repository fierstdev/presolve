#!/usr/bin/env bash
set -euo pipefail

repo_root="$(git rev-parse --show-toplevel)"
cd "$repo_root"

readonly contract=docs/specifications/phase-n/PHASE_N_N2A_TEMPLATE_INTERPOLATION_CONTRACT.md
test -s "$contract"
for phrase in \
  'untagged template literals' \
  'cooked literal segments' \
  'constructs one `Template` expression node' \
  'introduced runtime-computed artifact schema version `5`' \
  'never evaluates authored JavaScript source'; do
  rg --fixed-strings --quiet "$phrase" "$contract"
done

cargo test -q -p presolve-compiler \
  emits_template_interpolation_program_with_cooked_segments --lib
cargo test -q -p presolve-compiler \
  registry_is_versioned_stable_and_explains_deferred_families --lib
cargo test -q -p presolve-cli --test explain \
  build_command_writes_compiler_generated_computed_runtime_metadata
cargo test -q -p presolve-cli --test runtime_browser \
  template_interpolations_execute_from_compiler_generated_runtime_programs
cargo check -q -p presolve-compiler -p presolve-cli
cargo fmt --all --check
git diff --check

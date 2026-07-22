#!/usr/bin/env bash
set -euo pipefail

repo_root="$(git rev-parse --show-toplevel)"
cd "$repo_root"

readonly contract=docs/specifications/phase-n/PHASE_N_N1_MODULE_BINDINGS_CONTRACT.md
test -s "$contract"
for phrase in \
  'bounded semantic capability' \
  'does not discover files, install packages' \
  'advanced_types' \
  'PSBIND1001' \
  'does not change ASM, artifact, runtime, or resume schemas'; do
  rg --fixed-strings --quiet "$phrase" "$contract"
done

cargo test -q -p presolve-compiler \
  resolves_relative_named_default_and_namespace_imports --lib
cargo test -q -p presolve-compiler \
  resolves_named_and_export_all_reexport_chains --lib
cargo test -q -p presolve-compiler \
  registry_is_versioned_stable_and_explains_deferred_families --lib
cargo run -q -p presolve-cli -- explain --capabilities --format json \
  | rg --fixed-strings --quiet '"module_bindings"'
git diff --check

#!/usr/bin/env bash
set -euo pipefail

repo_root="$(git rev-parse --show-toplevel)"
cd "$repo_root"

readonly contract=docs/specifications/phase-n/PHASE_N_N1A_SEMANTIC_PACKAGE_CONTRACT.md
test -s "$contract"
for phrase in \
  'does not inspect package source' \
  'PSBIND1009' \
  'PSBIND1010' \
  'Namespace package imports and package re-exports are not admitted' \
  'does **not** make calling an imported package export compiler-native'; do
  rg --fixed-strings --quiet "$phrase" "$contract"
done

cargo test -q -p presolve-compiler \
  parses_integrity_checked_contract_and_resolves_an_export --lib
cargo test -q -p presolve-compiler \
  rejects_incomplete_contracts_without_replacing_existing_resolution --lib
cargo test -q -p presolve-compiler \
  resolves_external_imports_only_through_semantic_package_contracts --lib
cargo test -q -p presolve-compiler \
  rejects_external_imports_without_a_matching_contract_or_export --lib
cargo test -q -p presolve-compiler \
  resolves_relative_named_default_and_namespace_imports --lib
cargo test -q -p presolve-compiler \
  registry_is_versioned_stable_and_explains_deferred_families --lib
cargo run -q -p presolve-cli -- explain --capabilities --format json \
  | rg --fixed-strings --quiet '"semantic_package_bindings"'
cargo fmt --all --check
git diff --check

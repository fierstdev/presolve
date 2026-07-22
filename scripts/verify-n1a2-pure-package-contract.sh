#!/usr/bin/env bash
set -euo pipefail

repo_root="$(git rev-parse --show-toplevel)"
cd "$repo_root"

readonly contract=docs/specifications/phase-n/PHASE_N_N1A2_PURE_PACKAGE_CONTRACT.md
test -s "$contract"
for phrase in \
  'package JavaScript is neither loaded nor executed' \
  'pure-package-call' \
  '--package-contract value-kit=contracts/value-kit.json' \
  'PSBIND1009' \
  'compiler-owned'; do
  rg --fixed-strings --quiet -- "$phrase" "$contract"
done

cargo test -q -p presolve-compiler \
  emits_compiler_lowered_identity_package_call_with_contract_provenance --lib
cargo test -q -p presolve-compiler \
  resolves_external_imports_only_through_semantic_package_contracts --lib
cargo test -q -p presolve-cli --test explain \
  build_accepts_explicit_pure_package_contract_and_publishes_its_provenance
cargo test -q -p presolve-cli --test runtime_browser \
  pure_package_contracts_execute_only_the_compiler_lowered_operation_in_a_real_browser
cargo check -q -p presolve-compiler -p presolve-cli
cargo fmt --all --check
git diff --check

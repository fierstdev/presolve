#!/usr/bin/env bash
set -euo pipefail

repo_root="$(git rev-parse --show-toplevel)"
cd "$repo_root"

readonly contract=docs/specifications/phase-n/PHASE_N_N10_FREEZE_AND_FRAMEWORK_ADOPTION.md
test -s "$contract"
for phrase in \
  'Phase N freezes the compiler-owned semantic capability registry schema v1' \
  'The only admitted opaque form is' \
  'adapter, parser, package installer, or compatibility shim' \
  'pinned TypeScript 7.0 CLI'; do
  rg --fixed-strings --quiet "$phrase" "$contract"
done

for verifier in scripts/verify-n*.sh; do
  if [[ "$verifier" != "scripts/verify-n10-phase-n-freeze.sh" ]]; then
    "$verifier"
  fi
done
cargo test -p presolve-compiler semantic_capability::tests::registry_is_versioned_stable_and_explains_deferred_families -- --exact
RUST_TEST_THREADS=1 cargo test -p presolve-cli --test runtime_browser integrity_bound_opaque_terminal_runs_only_from_a_compiler_action_in_a_real_browser -- --nocapture
./scripts/verify-m2-framework-types.sh
cargo fmt --all --check
git diff --check

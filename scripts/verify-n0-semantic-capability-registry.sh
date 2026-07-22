#!/usr/bin/env bash
set -euo pipefail

repo_root="$(git rev-parse --show-toplevel)"
cd "$repo_root"

readonly contract=docs/specifications/phase-n/PHASE_N_N0_CAPABILITY_REGISTRY_CONTRACT.md
test -s "$contract"
for phrase in \
  'schema version `1`' \
  'semantic_package_exports' \
  'not a source-level escape hatch' \
  'Existing ASM, artifact, runtime, resume, and framework schema' \
  'does not run unrelated compiler suites'; do
  rg --fixed-strings --quiet "$phrase" "$contract"
done

cargo test -q -p presolve-compiler \
  registry_is_versioned_stable_and_explains_deferred_families --lib
cargo run -q -p presolve-cli -- explain --capabilities --format json \
  | rg --fixed-strings --quiet '"schema_version": 1'
cargo run -q -p presolve-cli -- explain --capabilities --format json \
  | rg --fixed-strings --quiet '"semantic_package_exports"'
cargo fmt --all --check
git diff --check

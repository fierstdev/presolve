#!/usr/bin/env bash
set -euo pipefail

repo_root="$(git rev-parse --show-toplevel)"
cd "$repo_root"

readonly plan=docs/specifications/phase-p/PHASE_P_MULTI_SOURCE_APPLICATION_PUBLICATION_PLAN.md
readonly contract=docs/specifications/phase-p/PHASE_P_P0_APPLICATION_PUBLICATION_CONSTITUTION.md
test -s "$plan"
test -s "$contract"
for phrase in \
  'explicit logical `entry_path`' \
  'ApplicationPublicationManifestV1' \
  'sibling staging directory' \
  'existing single-entry `presolve build <source>` command remains unchanged' \
  'may only project this command after P3'; do
  rg --fixed-strings --quiet "$phrase" "$plan" "$contract"
done
rg --fixed-strings --quiet 'O4 — multi-source artifact-publication decision' docs/specifications/phase-o/PHASE_O_APPLICATION_PRODUCTIZATION_PLAN.md
rg --fixed-strings --quiet 'Phase P' docs/specifications/phase-o/PHASE_O_APPLICATION_PRODUCTIZATION_PLAN.md
cargo fmt --all --check
git diff --check

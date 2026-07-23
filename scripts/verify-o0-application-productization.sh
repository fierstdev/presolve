#!/usr/bin/env bash
set -euo pipefail

repo_root="$(git rev-parse --show-toplevel)"
cd "$repo_root"

for document in \
  docs/specifications/phase-o/README.md \
  docs/specifications/phase-o/PHASE_O_APPLICATION_PRODUCTIZATION_PLAN.md \
  docs/specifications/phase-o/PHASE_O_O0_APPLICATION_PRODUCT_CONSTITUTION.md \
  docs/specifications/phase-o/PHASE_O_O1_EXPLICIT_APPLICATION_BUILD_HANDOFF.md; do
  test -s "$document"
done
for phrase in \
  'O0 — application product constitution' \
  'O1 — explicit application build handoff' \
  'O4 — multi-source artifact-publication decision' \
  'presolve build <entryPath> --out <outputDirectory>' \
  'must not read application source' \
  'single-entry artifact publisher'; do
  rg --fixed-strings --quiet "$phrase" docs/specifications/phase-o
done
rg --fixed-strings --quiet 'presolve build <file>' crates/presolve_cli/src/main.rs
rg --fixed-strings --quiet '"workspace" => run_l9_workspace' crates/presolve_cli/src/main.rs
rg --fixed-strings --quiet '"watch" => run_l9_watch' crates/presolve_cli/src/main.rs
git diff --check

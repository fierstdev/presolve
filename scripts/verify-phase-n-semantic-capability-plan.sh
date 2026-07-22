#!/usr/bin/env bash
set -euo pipefail

repo_root="$(git rev-parse --show-toplevel)"
cd "$repo_root"

readonly plan=docs/specifications/phase-n/PHASE_N_SEMANTIC_CAPABILITY_EXPANSION_PLAN.md
test -s "$plan"

for section in \
  'Objective' \
  'Governing principles' \
  'Capability classes' \
  'Third-party semantic package contracts' \
  'Developer capability target' \
  'Phase sequence' \
  'Required proof per capability' \
  'Explicit exclusions'; do
  rg --fixed-strings --quiet "$section" "$plan"
done

for slice in N0 N1 N2 N3 N4 N5 N6 N7 N8 N9 N10; do
  rg --fixed-strings --quiet "$slice" "$plan"
done

for phrase in \
  'Full-path admission' \
  'No implicit fallback' \
  'compiler-owned opaque-code escape hatch' \
  'not a generic escape hatch' \
  'package-manager installation and lockfile discovery remain outside compiler authority' \
  'no contract' \
  'Phase N does not promise arbitrary npm packages'; do
  rg --fixed-strings --quiet "$phrase" "$plan"
done

git diff --check

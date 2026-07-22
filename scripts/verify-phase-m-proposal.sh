#!/usr/bin/env bash
set -euo pipefail

roadmap=docs/specifications/phase-m/PHASE_M_PROPOSED_ROADMAP.md
test -s "$roadmap"
for heading in 'Product boundary' 'Architectural decisions' 'Slice sequence' 'Evidence matrix' 'Acceptance checklist'; do
  rg --fixed-strings --quiet "$heading" "$roadmap"
done
for slice in M0 M1 M2 M3 M4 M5 M6 M7 M8 M9; do
  rg --fixed-strings --quiet "$slice" "$roadmap"
done
for phrase in 'not implementation authority' 'reserved exit-6' 'No slice may begin' 'Presolve Metaframework' 'metaframework concern'; do
  rg --fixed-strings --quiet "$phrase" "$roadmap"
done
git diff --check

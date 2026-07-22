#!/usr/bin/env bash
set -euo pipefail

roadmap=docs/specifications/phase-m/PHASE_M_PROPOSED_ROADMAP.md
test -s "$roadmap"
for heading in 'Decision and scope' 'Compatibility and rollback' 'Slice sequence' 'Evidence matrix' 'Acceptance checklist'; do
  rg --fixed-strings --quiet "$heading" "$roadmap"
done
for slice in M0 M1 M2 M3 M4 M5 M6 M7; do
  rg --fixed-strings --quiet "$slice" "$roadmap"
done
for phrase in 'not implementation authority' 'reserved exit-6' 'No slice may begin' 'does not activate `dev`, `benchmark`, or `doctor`'; do
  rg --fixed-strings --quiet "$phrase" "$roadmap"
done
git diff --check

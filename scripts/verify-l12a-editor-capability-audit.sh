#!/usr/bin/env bash
set -euo pipefail
audit=docs/specifications/phase-l/PHASE_L_L12_EDITOR_CAPABILITY_AUDIT.md
test -s "$audit"
rg --quiet 'No current immutable public product is sufficient' "$audit"
rg --quiet 'L12-B must author the smallest immutable compiler-produced query snapshot' "$audit"
rg --quiet 'PHASE_L_L12_EDITOR_CAPABILITY_AUDIT' docs/specifications/phase-l/README.md
./scripts/verify-l11g-artifact-graph-command.sh
git diff --check

#!/usr/bin/env bash
set -euo pipefail

contract=docs/specifications/phase-l/PHASE_L_L11_TRACE_AND_COST_CONTRACT.md
test -s "$contract"
rg --quiet 'presolve.build-trace' "$contract"
rg --quiet 'presolve.compile-cost-report' "$contract"
rg --quiet 'wall-clock' "$contract"
rg --quiet 'must remain `reserved`' "$contract"
rg --quiet 'PHASE_L_L11_TRACE_AND_COST_CONTRACT' docs/specifications/phase-l/README.md
./scripts/verify-l11c-tooling-commands.sh
git diff --check

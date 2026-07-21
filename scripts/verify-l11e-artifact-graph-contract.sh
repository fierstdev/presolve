#!/usr/bin/env bash
set -euo pipefail

contract=docs/specifications/phase-l/PHASE_L_L11_ARTIFACT_GRAPH_CONTRACT.md
test -s "$contract"
rg --quiet 'presolve.artifact-graph' "$contract"
rg --quiet 'ProductionChunkGraph' "$contract"
rg --quiet 'must remain `reserved`' "$contract"
rg --quiet 'must not read, parse, glob, hash, or reconstruct' "$contract"
rg --quiet 'PHASE_L_L11_ARTIFACT_GRAPH_CONTRACT' docs/specifications/phase-l/README.md
./scripts/verify-l11d-trace-cost-contract.sh
git diff --check

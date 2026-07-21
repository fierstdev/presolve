#!/usr/bin/env bash
set -euo pipefail
contract=docs/specifications/phase-l/PHASE_L_L12_QUERY_SNAPSHOT_AMENDMENT.md
test -s "$contract"
rg --quiet 'presolve.query-snapshot' "$contract"
rg --quiet 'no authored source text' "$contract"
rg --quiet 'half-open UTF-8 byte range' "$contract"
rg --quiet 'QuerySemanticId' "$contract"
rg --quiet 'Provenance-free internal or synthesized entities' "$contract"
rg --quiet 'PHASE_L_L12_QUERY_SNAPSHOT_AMENDMENT' docs/specifications/phase-l/README.md
./scripts/verify-l12a-editor-capability-audit.sh
git diff --check

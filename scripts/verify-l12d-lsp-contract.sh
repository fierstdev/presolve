#!/usr/bin/env bash
set -euo pipefail
contract=docs/specifications/phase-l/PHASE_L_L12_LSP_CONTRACT.md
test -s "$contract"
rg --quiet 'LSP adapter contract' "$contract"
rg --quiet 'stateless translation layer' "$contract"
rg --quiet 'unsupported' "$contract"
rg --quiet 'PHASE_L_L12_LSP_CONTRACT' docs/specifications/phase-l/README.md
./scripts/verify-l12c4-language-service.sh
git diff --check

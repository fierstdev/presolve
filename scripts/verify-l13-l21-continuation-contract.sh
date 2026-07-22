#!/usr/bin/env bash
set -euo pipefail
contract=docs/specifications/phase-l/PHASE_L_L13_L21_CONTINUATION_CONTRACT.md
test -s "$contract"
rg --quiet 'The required order is L15, L14, L13, L16, L17, L18, L19, L20, then L21' "$contract"
rg --quiet 'No slice may parse, bind, analyze, or diagnose authored source' "$contract"
rg --quiet '## L13 -- tested public documentation' "$contract"
rg --quiet '## L20 -- platform freeze' "$contract"
rg --quiet '## L21 -- post-freeze stewardship handoff' "$contract"
rg --quiet 'PHASE_L_L13_L21_CONTINUATION_CONTRACT' docs/specifications/phase-l/README.md
git diff --check

#!/usr/bin/env bash
set -euo pipefail
contract=docs/examples-contract.md
test -s "$contract"
rg --quiet 'Presolve alpha example contract' "$contract"
rg --quiet 'Counter' "$contract"
rg --quiet 'Components/Context/Slots' "$contract"
rg --quiet 'Forms' "$contract"
rg --quiet 'Explicit workspace' "$contract"
rg --quiet 'Production/resume' "$contract"
rg --quiet 'exactly these five examples' "$contract"
./scripts/verify-l15c-reproducibility-lanes.sh
git diff --check

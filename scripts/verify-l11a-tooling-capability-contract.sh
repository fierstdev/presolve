#!/usr/bin/env bash
set -euo pipefail

contract=docs/specifications/phase-l/PHASE_L_L11_TOOLING_CAPABILITY_CONTRACT.md

test -s "$contract"
rg --quiet 'L11 turns existing immutable compiler-platform facts' "$contract"
rg --quiet 'presolve <tool> --schema <registered-schema> --product <caller-named-file>' "$contract"
rg --quiet 'L11T001' "$contract"
rg --quiet 'L11T006' "$contract"
rg --quiet 'presolve.build-trace' "$contract"
rg --quiet 'L12-A capability audit' "$contract"
rg --quiet 'PHASE_L_L11_TOOLING_CAPABILITY_CONTRACT' docs/specifications/phase-l/README.md
./scripts/verify-l10-schema-contract.sh
git diff --check

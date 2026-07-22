#!/usr/bin/env bash
set -euo pipefail
contract=docs/testing-contract.md
test -s "$contract"
rg --quiet 'L15-A authoritative inventory' "$contract"
rg --quiet 'Compiler/platform' "$contract"
rg --quiet 'Tooling/products' "$contract"
rg --quiet 'Runtime/browser' "$contract"
rg --quiet 'can never affect a pass/fail result' "$contract"
rg --quiet 'public testing contract' docs/specifications/phase-l/README.md
./scripts/verify-l13-l21-continuation-contract.sh
git diff --check

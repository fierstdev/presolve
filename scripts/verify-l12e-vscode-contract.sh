#!/usr/bin/env bash
set -euo pipefail
contract=docs/specifications/phase-l/PHASE_L_L12_VSCODE_CONTRACT.md
test -s "$contract"
rg --quiet '@presolve/vscode' "$contract"
rg --quiet 'depends exclusively on `@presolve/lsp`' "$contract"
rg --quiet 'stable LSP unsupported result' "$contract"
rg --quiet 'PHASE_L_L12_VSCODE_CONTRACT' docs/specifications/phase-l/README.md
./scripts/verify-l12d2-lsp-adapter.sh
git diff --check

#!/usr/bin/env bash
set -euo pipefail

contract=docs/specifications/phase-l/PHASE_L_L12_WASM_BINDING_CONTRACT.md
test -s "$contract"
rg --quiet 'Compiler-owned WASM language-service binding contract' "$contract"
rg --quiet 'query_snapshot_v1' "$contract"
rg --quiet 'decode_tooling_query_snapshot_v1' "$contract"
rg --quiet 'no JavaScript product decoder' "$contract"
rg --quiet 'invalid_product' "$contract"
rg --quiet 'unsupported' "$contract"
rg --quiet '@presolve/compiler-wasm' "$contract"
rg --quiet 'PHASE_L_L12_WASM_BINDING_CONTRACT' docs/specifications/phase-l/README.md
./scripts/verify-l12c-language-service-binding-audit.sh
git diff --check

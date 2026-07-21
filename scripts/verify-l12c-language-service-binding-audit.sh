#!/usr/bin/env bash
set -euo pipefail

audit=docs/specifications/phase-l/PHASE_L_L12_LANGUAGE_SERVICE_BINDING_AUDIT.md
test -s "$audit"
rg --quiet '@presolve/language-service' "$audit"
rg --quiet 'decode_tooling_query_snapshot_v1' "$audit"
rg --quiet 'WASM ABI' "$audit"
rg --quiet 'native addon' "$audit"
rg --quiet 'Rust-native language-service API' "$audit"
rg --quiet 'PHASE_L_L12_LANGUAGE_SERVICE_BINDING_AUDIT' docs/specifications/phase-l/README.md
./scripts/verify-l12c-query-snapshot-product.sh
git diff --check

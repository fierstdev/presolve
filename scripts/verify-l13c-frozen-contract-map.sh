#!/usr/bin/env bash
set -euo pipefail
map=docs/frozen-contract-map.md
test -s "$map"
rg --quiet '^# Frozen contract map$' "$map"
for subject in State Actions Computed Context Components Slots Forms Resumability Production/runtime Service Cache Workspace 'L12 language service' 'L12 LSP' 'L12 VSCode facade'; do
  rg --fixed-strings --quiet "| $subject |" "$map"
done
for authority in runtime-contract.md context-contract.md component-contract.md forms-contract.md resumability-contract.md production-optimization-contract.md compiler-service-contract.md persistent-cache-contract.md workspace-architecture-contract.md PHASE_L_L12_WASM_BINDING_CONTRACT.md PHASE_L_L12_LSP_CONTRACT.md PHASE_L_L12_VSCODE_CONTRACT.md; do
  rg --quiet "$authority" "$map"
done
./scripts/verify-l13b-public-cli-docs.sh
git diff --check

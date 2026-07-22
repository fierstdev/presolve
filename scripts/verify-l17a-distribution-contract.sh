#!/usr/bin/env bash
set -euo pipefail
contract=docs/distribution-contract.md
test -s "$contract"
for package in compiler-wasm language-service lsp vscode testing runtime; do
  manifest="packages/$package/package.json"
  rg --quiet '"private": true' "$manifest"
done
for package in @presolve/compiler-wasm @presolve/language-service @presolve/lsp @presolve/vscode @presolve/testing @presolve/runtime; do
  rg --fixed-strings --quiet "$package" "$contract"
done
rg --quiet 'compiler-wasm → language-service → lsp → vscode' "$contract"
rg --quiet -- '--offline' "$contract"
rg --quiet 'pnpm -r check' "$contract"
rg --quiet 'No package in this repository is publishable' "$contract"
./scripts/verify-l16-community-readiness.sh
git diff --check

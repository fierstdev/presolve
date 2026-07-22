#!/usr/bin/env bash
set -euo pipefail
package=packages/vscode
pnpm install --offline
cleanup_workspace_links() {
  rm -rf packages/language-service/node_modules packages/lsp/node_modules packages/vscode/node_modules
}
trap cleanup_workspace_links EXIT
rg --quiet '@presolve/lsp' "$package/package.json"
rg --quiet 'initializeLsp' "$package/src/index.js"
if rg --quiet '@presolve/language-service|@presolve/compiler-wasm|fs|fetch|http|cache|persist|vscode|TextDocument|Workspace|edit' "$package/src/index.js"; then
  echo 'L12-E facade must depend only on LSP and own no editor/product authority' >&2
  exit 1
fi
./scripts/build-l12c-compiler-wasm.sh
node "$package/test/pinned-editor-fixture.mjs"
./scripts/verify-l12d2-lsp-adapter.sh
git diff --check

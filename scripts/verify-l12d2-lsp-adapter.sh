#!/usr/bin/env bash
set -euo pipefail
package=packages/lsp
pnpm install --offline --force
cleanup_workspace_links() {
  rm -rf packages/language-service/node_modules packages/lsp/node_modules packages/vscode/node_modules
}
trap cleanup_workspace_links EXIT
rg --quiet '@presolve/language-service' "$package/package.json"
rg --quiet 'textDocument/definition' "$package/src/index.js"
rg --quiet 'textDocument/references' "$package/src/index.js"
rg --quiet 'unsupported' "$package/src/index.js"
if rg --quiet 'node:fs|from "fs"|fetch\(|http|cache|persist|didChange|didOpen|decode_tooling_query_snapshot' "$package/src/index.js"; then
  echo 'L12-D adapter must remain stateless and product-only' >&2
  exit 1
fi
./scripts/build-l12c-compiler-wasm.sh
node "$package/test/smoke.mjs"
./scripts/verify-l12c4-language-service.sh
git diff --check

#!/usr/bin/env bash
set -euo pipefail

package=packages/language-service
test -s "$package/package.json"
test -s "$package/src/index.js"
rg --quiet '@presolve/compiler-wasm' "$package/package.json"
rg --quiet 'query_snapshot_v1' "$package/src/index.js"
if rg --quiet 'decode_tooling_query_snapshot|serde|fs|fetch|http|cache|persist|sourceText' "$package/src/index.js"; then
  echo 'L12-C-4 language service must remain a thin WASM-only wrapper' >&2
  exit 1
fi
./scripts/build-l12c-compiler-wasm.sh
node "$package/test/smoke.mjs"
./scripts/verify-l12c3-wasm-binding.sh
git diff --check

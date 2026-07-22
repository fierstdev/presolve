#!/usr/bin/env bash
set -euo pipefail
package=packages/testing
test -s "$package/package.json"
rg --quiet 'equalCanonicalBytes' "$package/src/index.js"
rg --quiet 'declaredTest' "$package/src/index.js"
if rg --quiet 'fs|fetch|http|child_process|spawn|exec\(|decode|parser|compiler|browser|performance' "$package/src/index.js"; then
  echo 'L15-B package must remain a pure test utility' >&2
  exit 1
fi
node "$package/test/smoke.mjs"
./scripts/verify-l15a-testing-contract.sh
git diff --check

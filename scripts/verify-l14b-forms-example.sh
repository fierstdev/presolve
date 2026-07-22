#!/usr/bin/env bash
set -euo pipefail
example=examples/forms
cleanup() { rm -rf "$example/.presolve"; }
trap cleanup EXIT
test -s "$example/presolve.json"
test -s "$example/src/Forms.tsx"
result="$(cargo run -q -p presolve-cli -- check --config "$example/presolve.json" --source Forms.tsx=src/Forms.tsx)"
printf '%s\n' "$result" | rg --quiet '"status":"ok"'
./scripts/verify-l14b-components-context-slots-example.sh
git diff --check

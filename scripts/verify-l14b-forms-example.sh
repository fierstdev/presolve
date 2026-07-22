#!/usr/bin/env bash
set -euo pipefail
example=examples/forms
cleanup() { rm -rf "$example/.presolve"; }
trap cleanup EXIT
test -s "$example/presolve.json"
test -s "$example/src/Forms.tsx"
result="$(cargo run -q -p presolve-cli -- check --config "$example/presolve.json" --source Forms.tsx=src/Forms.tsx --format json)"
printf '%s\n' "$result" | rg --quiet '"status":"succeeded"'
./scripts/verify-l14b-components-context-slots-example.sh
git diff --check

#!/usr/bin/env bash
set -euo pipefail
example=examples/explicit-workspace
cleanup() { rm -rf "$example/.presolve"; }
trap cleanup EXIT
test -s "$example/presolve.json"
test -s "$example/src/main.ts"
result="$(cargo run -q -p presolve-cli -- workspace --config "$example/presolve.json" --source src/main.ts=src/main.ts --format json)"
printf '%s\n' "$result" | rg --quiet '"schema":"presolve.cli-workspace-result"'
printf '%s\n' "$result" | rg --quiet '"status":"succeeded"'
./scripts/verify-l14b-forms-example.sh
git diff --check

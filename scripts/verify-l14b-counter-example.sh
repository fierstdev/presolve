#!/usr/bin/env bash
set -euo pipefail
example=examples/counter
cleanup() { rm -rf "$example/.presolve"; }
trap cleanup EXIT
test -s "$example/presolve.json"
test -s "$example/src/Counter.tsx"
result="$(cargo run -q -p presolve-cli -- check --config "$example/presolve.json" --source counter.tsx=src/Counter.tsx --format json)"
printf '%s\n' "$result" | rg --quiet '"status":"succeeded"'
./scripts/verify-l14a-examples-contract.sh
git diff --check

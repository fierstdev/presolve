#!/usr/bin/env bash
set -euo pipefail
example=examples/components-context-slots
cleanup() { rm -rf "$example/.presolve"; }
trap cleanup EXIT
test -s "$example/presolve.json"
test -s "$example/src/Composition.tsx"
result="$(cargo run -q -p presolve-cli -- check --config "$example/presolve.json" --source Composition.tsx=src/Composition.tsx --format json)"
printf '%s\n' "$result" | rg --quiet '"status":"succeeded"'
./scripts/verify-l14b-counter-example.sh
git diff --check

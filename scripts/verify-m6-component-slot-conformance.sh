#!/usr/bin/env bash
set -euo pipefail

repo_root="$(git rev-parse --show-toplevel)"
cd "$repo_root"

readonly contract=docs/specifications/phase-m/PHASE_M_M6_COMPONENT_SLOT_CONFORMANCE.md
readonly fixture=framework/tests/composition-types
readonly compiler_fixture=fixtures/0062-component-declarations/input/ValidComponents.tsx

test -s "$contract"
for phrase in 'SlotContent' '@slot()' 'byte-identical' 'Slot-binding programs' 'never a framework shim'; do
  rg --fixed-strings --quiet "$phrase" "$contract"
done
test -s "$fixture/tsconfig.json"
test -s "$fixture/presolve.json"
test -s "$fixture/src/ValidComponents.tsx"
rg --fixed-strings --quiet 'function slot()' framework/packages/framework-types/src/index.d.ts
rg --fixed-strings --quiet 'interface SlotContent' framework/packages/framework-types/src/index.d.ts
cmp -- "$fixture/src/ValidComponents.tsx" "$compiler_fixture"

pnpm exec tsc --project "$fixture/tsconfig.json"
rm -rf "$fixture/.presolve"
trap 'rm -rf "$fixture/.presolve"' EXIT
cargo run -q -p presolve-cli -- check \
  --config "$fixture/presolve.json" \
  --source components.tsx=src/ValidComponents.tsx \
  --format json | rg --fixed-strings --quiet '"status":"succeeded"'
cargo test -q -p presolve-cli --test runtime_browser \
  component_runtime_consumes_only_compiler_plans_in_a_real_browser -- --nocapture
git diff --check

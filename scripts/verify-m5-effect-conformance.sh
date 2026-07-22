#!/usr/bin/env bash
set -euo pipefail

repo_root="$(git rev-parse --show-toplevel)"
cd "$repo_root"

readonly contract=docs/specifications/phase-m/PHASE_M_M5_EFFECT_CONFORMANCE.md
readonly fixture=framework/tests/effect-types
readonly compiler_fixture=fixtures/0053-effect-initial-runtime/input/InitialEffectRuntime.tsx

test -s "$contract"
for phrase in '@effect()' 'byte-identical' 'single initial run' 'exact compiler capability-dispatch order' 'generic `useEffect` analogue'; do
  rg --fixed-strings --quiet "$phrase" "$contract"
done
test -s "$fixture/tsconfig.json"
test -s "$fixture/presolve.json"
test -s "$fixture/src/InitialEffectRuntime.tsx"
rg --fixed-strings --quiet 'function effect()' framework/packages/framework-types/src/index.d.ts
cmp -- "$fixture/src/InitialEffectRuntime.tsx" "$compiler_fixture"

pnpm exec tsc --project "$fixture/tsconfig.json"
rm -rf "$fixture/.presolve"
trap 'rm -rf "$fixture/.presolve"' EXIT
cargo run -q -p presolve-cli -- check \
  --config "$fixture/presolve.json" \
  --source effects.tsx=src/InitialEffectRuntime.tsx \
  --format json | rg --fixed-strings --quiet '"status":"succeeded"'
cargo test -q -p presolve-cli --test runtime_browser \
  initial_effects_execute_once_from_compiler_generated_runtime_programs -- --nocapture
git diff --check

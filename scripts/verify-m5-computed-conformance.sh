#!/usr/bin/env bash
set -euo pipefail

repo_root="$(git rev-parse --show-toplevel)"
cd "$repo_root"

readonly contract=docs/specifications/phase-m/PHASE_M_M5_COMPUTED_CONFORMANCE.md
readonly fixture=framework/tests/computed-types
readonly example=examples/production-resume/src/ComputedDiamond.tsx
readonly compiler_fixture=fixtures/0047-computed-diamond/input/ComputedDiamond.tsx

test -s "$contract"
for phrase in '@computed()' 'byte-identical' 'no JavaScript framework package' 'single compiler-planned update'; do
  rg --fixed-strings --quiet "$phrase" "$contract"
done
test -s "$fixture/tsconfig.json"
test -s "$fixture/presolve.json"
test -s "$fixture/src/ComputedDiamond.tsx"
rg --fixed-strings --quiet 'function computed()' framework/packages/framework-types/src/index.d.ts
cmp -- "$fixture/src/ComputedDiamond.tsx" "$example"
cmp -- "$fixture/src/ComputedDiamond.tsx" "$compiler_fixture"

pnpm exec tsc --project "$fixture/tsconfig.json"
rm -rf "$fixture/.presolve"
trap 'rm -rf "$fixture/.presolve"' EXIT
cargo run -q -p presolve-cli -- check \
  --config "$fixture/presolve.json" \
  --source computed.tsx=src/ComputedDiamond.tsx \
  --format json | rg --fixed-strings --quiet '"status":"succeeded"'
cargo test -q -p presolve-cli --test runtime_browser \
  diamond_computed_values_recompute_from_compiler_generated_batches -- --nocapture
git diff --check

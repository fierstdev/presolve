#!/usr/bin/env bash
set -euo pipefail

repo_root="$(git rev-parse --show-toplevel)"
cd "$repo_root"

readonly contract=docs/specifications/phase-m/PHASE_M_M6_CONTEXT_LANGUAGE_CONTRACT.md
readonly fixture=framework/tests/context-types
readonly compiler_fixture=fixtures/0059-context-runtime-matrix/input/ContextRuntimeMatrix.tsx

test -s "$contract"
for phrase in \
  'Context is the one Phase M composition family' \
  'Identifier.Identifier' \
  'compile-time syntax, not a dynamic string key' \
  'not framework API' \
  'byte-identical'; do
  rg --fixed-strings --quiet "$phrase" "$contract"
done

test -s "$fixture/tsconfig.json"
test -s "$fixture/presolve.json"
test -s "$fixture/src/ContextRuntimeMatrix.tsx"
rg --fixed-strings --quiet 'type ContextDesignator' framework/packages/framework-types/src/index.d.ts
rg --fixed-strings --quiet 'function context()' framework/packages/framework-types/src/index.d.ts
rg --fixed-strings --quiet 'function provide(contextDesignator: ContextDesignator)' framework/packages/framework-types/src/index.d.ts
rg --fixed-strings --quiet 'function consume(contextDesignator: ContextDesignator)' framework/packages/framework-types/src/index.d.ts
cmp -- "$fixture/src/ContextRuntimeMatrix.tsx" "$compiler_fixture"

CI=true pnpm exec tsc --project "$fixture/tsconfig.json"
cargo test -q -p presolve-compiler \
  accepts_static_contexts_and_qualified_string_designators --lib
cargo test -q -p presolve-compiler \
  retains_invalid_context_candidates_for_g18_diagnostics --lib
rm -rf "$fixture/.presolve"
trap 'rm -rf "$fixture/.presolve"' EXIT
cargo run -q -p presolve-cli -- check \
  --config "$fixture/presolve.json" \
  --source context.tsx=src/ContextRuntimeMatrix.tsx \
  --format json | rg --fixed-strings --quiet '"status":"succeeded"'
cargo test -q -p presolve-cli --test runtime_browser \
  context_sources_bind_and_update_from_compiler_plans_in_a_real_browser -- --nocapture
git diff --check

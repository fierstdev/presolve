#!/usr/bin/env bash
set -euo pipefail

repo_root="$(git rev-parse --show-toplevel)"
cd "$repo_root"

readonly package=framework/packages/framework-types
readonly fixture=framework/tests/counter-types

test -s "$package/package.json"
test -s "$package/src/index.d.ts"
test -s "$fixture/src/Counter.tsx"
test -s "$fixture/src/Opaque.tsx"
test -s "$fixture/presolve.json"
test -s "$fixture/tsconfig.json"

rg --fixed-strings --quiet '"name": "@presolve/framework-types"' "$package/package.json"
rg --fixed-strings --quiet '"private": true' "$package/package.json"
rg --fixed-strings --quiet '"types": "./src/index.d.ts"' "$package/package.json"
rg --fixed-strings --quiet 'abstract class Component' "$package/src/index.d.ts"
rg --fixed-strings --quiet 'function component(elementName: string)' "$package/src/index.d.ts"
rg --fixed-strings --quiet 'function state<T>(initialValue: T): T' "$package/src/index.d.ts"
rg --fixed-strings --quiet 'function opaque(packageSpecifier: string, exportName: string)' "$package/src/index.d.ts"
rg --fixed-strings --quiet '"types": [' "$fixture/tsconfig.json"
rg --fixed-strings --quiet '"@presolve/framework-types"' "$fixture/tsconfig.json"

if find "$package/src" -type f ! -name '*.d.ts' -print | grep -q .; then
  echo 'M2 framework types must emit no JavaScript source' >&2
  exit 1
fi

if rg --line-number 'jsx-runtime|jsx\(|createElement|Proxy|Map<|Set<|fetch|fs|child_process|spawn|exec\(' "$package"; then
  echo 'M2 framework types must remain declaration-only and runtime-free' >&2
  exit 1
fi

cmp -- "$fixture/src/Counter.tsx" examples/counter/src/Counter.tsx
typescript_version="$(pnpm exec tsc --version)"
if [[ ! "$typescript_version" =~ (^|$'\n')Version\ 7\.0\.[0-9]+($|$'\n') ]]; then
  echo "M2 requires the pinned TypeScript 7.0 native CLI, found: $typescript_version" >&2
  exit 1
fi
pnpm exec tsc --project "$fixture/tsconfig.json"

cleanup() { rm -rf "$fixture/.presolve"; }
trap cleanup EXIT
result="$(cargo run -q -p presolve-cli -- check --config "$fixture/presolve.json" --source counter.tsx=src/Counter.tsx --format json)"
printf '%s\n' "$result" | rg --fixed-strings --quiet '"status":"succeeded"'

./scripts/verify-phase-m-proposal.sh
git diff --check

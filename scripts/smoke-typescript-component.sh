#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
EXAMPLE="$ROOT/examples/typescript-component-checkpoint"
PRESOLVE="$ROOT/target/debug/presolve"
TMP="$(mktemp -d "${TMPDIR:-/tmp}/presolve-ts-component.XXXXXX")"

cleanup() {
  rm -rf "$TMP"
}
trap cleanup EXIT

echo "== build TypeScript SDK =="
pnpm --dir "$ROOT" --filter @presolve/sdk build

echo "== type-check TypeScript Component example =="
pnpm --dir "$ROOT" --filter @presolve/example-typescript-component-checkpoint typecheck

echo "== build Presolve CLI =="
cargo build --manifest-path "$ROOT/Cargo.toml" -p presolve-cli

build_checkpoint() {
  rm -rf "$EXAMPLE/.presolve" "$EXAMPLE/dist"

  (
    cd "$EXAMPLE"
    "$PRESOLVE" build
  )
}

echo "== first TypeScript → Component → .presolved build =="
build_checkpoint

COMPONENT="$EXAMPLE/.presolve/build/typescript/main.wasm"
RELEASE="$EXAMPLE/dist/typescript-component-checkpoint.presolved"
WIT_OUTPUT="$EXAMPLE/.presolve/build/typescript/component.wit.txt"

test -f "$COMPONENT"
test -f "$RELEASE"

cp "$COMPONENT" "$TMP/first.wasm"
cp "$RELEASE" "$TMP/first.presolved"

echo "== second clean TypeScript build =="
build_checkpoint

test -f "$COMPONENT"
test -f "$RELEASE"

echo "== compare TypeScript build outputs =="
cmp "$TMP/first.wasm" "$COMPONENT"
cmp "$TMP/first.presolved" "$RELEASE"

echo "== inspect generated Component WIT =="
(
  cd "$EXAMPLE"
  pnpm exec jco wit ".presolve/build/typescript/main.wasm"
) > "$WIT_OUTPUT"

grep -q 'presolve:kv/store@0.1.0' "$WIT_OUTPUT"
grep -q 'run: func' "$WIT_OUTPUT"

echo "TypeScript Component checkpoint passed"

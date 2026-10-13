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

component_wit() {
  local component="$1"
  local output="$2"

  (
    cd "$EXAMPLE"
    pnpm exec jco wit "$component"
  ) > "$output"
}

echo "== first TypeScript → Component → .presolved build =="
build_checkpoint

BUILD_DIRECTORY="$EXAMPLE/.presolve/build/typescript"
COMPONENT="$BUILD_DIRECTORY/main.wasm"
RELEASE="$EXAMPLE/dist/typescript-component-checkpoint.presolved"
BOOTSTRAP="$BUILD_DIRECTORY/bootstrap.ts"
WIT_DIRECTORY="$BUILD_DIRECTORY/wit"

test -f "$COMPONENT"
test -f "$RELEASE"
test -f "$BOOTSTRAP"
test -d "$WIT_DIRECTORY"

cp "$COMPONENT" "$TMP/first.wasm"
cp "$RELEASE" "$TMP/first.presolved"
cp "$BOOTSTRAP" "$TMP/first-bootstrap.ts"
cp -R "$WIT_DIRECTORY" "$TMP/first-wit"
component_wit "$TMP/first.wasm" "$TMP/first-component.wit"

echo "== second clean TypeScript build =="
build_checkpoint

test -f "$COMPONENT"
test -f "$RELEASE"
test -f "$BOOTSTRAP"
test -d "$WIT_DIRECTORY"

echo "== compare Presolve-generated TypeScript build inputs =="
cmp "$TMP/first-bootstrap.ts" "$BOOTSTRAP"
diff -ru "$TMP/first-wit" "$WIT_DIRECTORY"

echo "== compare generated Component interface =="
component_wit "$COMPONENT" "$TMP/second-component.wit"
cmp "$TMP/first-component.wit" "$TMP/second-component.wit"
grep -q 'presolve:kv/store@0.1.0' "$TMP/second-component.wit"
grep -q 'run: func' "$TMP/second-component.wit"

echo "== observe external source-toolchain byte reproducibility =="
if cmp -s "$TMP/first.wasm" "$COMPONENT"; then
  echo "Jco/ComponentizeJS emitted byte-identical Components on this platform."
  cmp "$TMP/first.presolved" "$RELEASE"
else
  echo "Jco/ComponentizeJS emitted different Component bytes across clean builds."
  echo "This is external source-toolchain variance, not a Presolve packaging-determinism failure."
fi

echo "== package one generated Component artifact twice =="
(
  cd "$EXAMPLE"
  "$PRESOLVE" build \
    --component "$TMP/first.wasm" \
    --output "$TMP/repack-first.presolved" > /dev/null
  "$PRESOLVE" build \
    --component "$TMP/first.wasm" \
    --output "$TMP/repack-second.presolved" > /dev/null
)

cmp "$TMP/repack-first.presolved" "$TMP/repack-second.presolved"

echo "TypeScript Component checkpoint passed"

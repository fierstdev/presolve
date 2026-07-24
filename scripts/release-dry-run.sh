#!/usr/bin/env bash
set -euo pipefail
temporary_dir="$(mktemp -d)"
cleanup() { rm -rf "$temporary_dir"; }
trap cleanup EXIT
pnpm install --frozen-lockfile
node scripts/verify-release-version.mjs
# Build the compiler-owned query binding once, then exercise each public
# projection without treating historical implementation-check scripts as the
# public release authority.
./scripts/build-l12c-compiler-wasm.sh
node packages/compiler-wasm/test/smoke.mjs
node packages/language-service/test/smoke.mjs
node packages/lsp/test/smoke.mjs
node packages/vscode/test/pinned-editor-fixture.mjs
node packages/testing/test/smoke.mjs
pnpm --dir packages/create-presolve test
node_modules/.bin/tsc -p framework/tests/public-package-types/tsconfig.json --pretty false
cargo build -p presolve-cli
./scripts/verify-r9-usability-freeze.sh
printf '{"schema":"presolve.release-dry-run","version":1,"packages":['
first=true
for package in framework/packages/presolve packages/cli packages/create-presolve packages/compiler-wasm packages/language-service packages/lsp packages/testing metaframework/packages/application packages/vscode; do
  packed="$(pnpm --dir "$package" pack --json --pack-destination "$temporary_dir")"
  tarball="$(node --input-type=module -e 'const value=JSON.parse(process.argv[1]); console.log(value.tarball ?? value.filename);' "$packed")"
  checksum="$(shasum -a 256 "$tarball" | awk '{print $1}')"
  name="$(node --input-type=module -e 'const value=JSON.parse(process.argv[1]); console.log(value.name);' "$packed")"
  version="$(node --input-type=module -e 'const value=JSON.parse(process.argv[1]); console.log(value.version);' "$packed")"
  if "$first"; then first=false; else printf ','; fi
  printf '{"name":"%s","version":"%s","sha256":"%s"}' "$name" "$version" "$checksum"
done
printf ']}\n'

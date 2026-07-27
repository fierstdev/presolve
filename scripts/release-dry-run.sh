#!/usr/bin/env bash
set -euo pipefail

release_dir="$(mktemp -d)"
cleanup() { git clean -fd -- packages/cli-*/bin >/dev/null 2>&1 || true; rm -rf "$release_dir"; }
trap cleanup EXIT

pnpm install --frozen-lockfile
node scripts/verify-release-version.mjs
pnpm run check
pnpm run test:scaffold
# The parser is packaged locally. The compiler and CLI package after that
# parser version is visible on crates.io; the release workflow performs those
# dependency-ordered publishes with retry rather than pretending a first,
# unpublished beta can resolve from the public registry during a dry run.
cargo package -p presolve-parser --allow-dirty --no-verify
pnpm run release:prepare
(
  cd packages/vscode
  npm exec --yes --package=@vscode/vsce@3.9.2 -- vsce package \
    --no-dependencies \
    --pre-release \
    --out "$release_dir/presolve-vscode.vsix"
)

printf '{"schema":"presolve.release-dry-run","version":1,"packages":['
first=true
for package in framework/packages/presolve packages/cli packages/create-presolve packages/compiler-wasm packages/language-service packages/lsp packages/testing packages/typescript-authority packages/vite packages/vscode; do
  packed="$(pnpm --dir "$package" pack --json --pack-destination "$release_dir")"
  tarball="$(node --input-type=module -e 'const value=JSON.parse(process.argv[1]); console.log(value.tarball ?? value.filename);' "$packed")"
  checksum="$(shasum -a 256 "$tarball" | awk '{print $1}')"
  name="$(node --input-type=module -e 'const value=JSON.parse(process.argv[1]); console.log(value.name);' "$packed")"
  version="$(node --input-type=module -e 'const value=JSON.parse(process.argv[1]); console.log(value.version);' "$packed")"
  if "$first"; then first=false; else printf ','; fi
  printf '{"name":"%s","version":"%s","sha256":"%s"}' "$name" "$version" "$checksum"
done
printf ']}\n'

#!/usr/bin/env bash
set -euo pipefail
temporary_dir="$(mktemp -d)"
cleanup() { rm -rf "$temporary_dir"; }
trap cleanup EXIT
pnpm install --offline --force
# Existing package verifiers resolve repository-root-relative compiler fixtures;
# execute their root-owned entry points instead of package-local pnpm scripts.
./scripts/verify-l12c3-wasm-binding.sh
./scripts/verify-l12c4-language-service.sh
./scripts/verify-l12d2-lsp-adapter.sh
./scripts/verify-l12e2-vscode-facade.sh
./scripts/verify-l15b-testing-package.sh
pnpm --dir packages/runtime run check
printf '{"schema":"presolve.release-dry-run","version":1,"packages":['
first=true
for package in packages/compiler-wasm packages/language-service packages/lsp packages/runtime packages/testing packages/vscode; do
  packed="$(pnpm --dir "$package" pack --json --pack-destination "$temporary_dir")"
  tarball="$(node --input-type=module -e 'const value=JSON.parse(process.argv[1]); console.log(value.tarball);' "$packed")"
  checksum="$(shasum -a 256 "$tarball" | awk '{print $1}')"
  name="$(node --input-type=module -e 'const value=JSON.parse(process.argv[1]); console.log(value.name);' "$packed")"
  version="$(node --input-type=module -e 'const value=JSON.parse(process.argv[1]); console.log(value.version);' "$packed")"
  if "$first"; then first=false; else printf ','; fi
  printf '{"name":"%s","version":"%s","sha256":"%s"}' "$name" "$version" "$checksum"
done
printf ']}\n'

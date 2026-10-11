#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

for command in pnpm grep; do
  if ! command -v "$command" >/dev/null 2>&1; then
    echo "missing required command: $command" >&2
    exit 1
  fi
done

echo "== TypeScript SDK typecheck =="
pnpm --filter @presolve/sdk typecheck

echo "== TypeScript SDK tests =="
pnpm --filter @presolve/sdk test

echo "== TypeScript OCI example typecheck =="
pnpm --filter @presolve/example-typescript-container-checkpoint typecheck

echo "== authoring-boundary assertions =="

APPLICATION_SURFACE=(
  packages/sdk/src/index.ts
  packages/sdk/src/kv.ts
  packages/sdk/src/objects.ts
)

if grep -nE \
  'PRESOLVE_CAPABILITY_|host\.docker\.internal|ContainerBridgeTransport|ComponentTransport|ProviderId' \
  "${APPLICATION_SURFACE[@]}"; then
  echo "application-facing TypeScript SDK leaked execution/provider details" >&2
  exit 1
fi

grep -q '"./container"' packages/sdk/package.json
grep -q '"./component"' packages/sdk/package.json
grep -q 'presolve:kv/store' packages/sdk/src/kv.ts
grep -q 'presolve:objects/store' packages/sdk/src/objects.ts
grep -q 'presolve-http-json/0.1' packages/sdk/src/container.ts
grep -q 'ComponentObjectStoreImports' packages/sdk/src/component.ts

echo "TypeScript authoring checkpoint passed"
echo "For the real OCI execution proof, run: pnpm smoke:ts-oci"

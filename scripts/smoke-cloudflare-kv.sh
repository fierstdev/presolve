#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

required=(
  PRESOLVE_CLOUDFLARE_ACCOUNT_ID
  PRESOLVE_CLOUDFLARE_KV_NAMESPACE_ID
  PRESOLVE_CLOUDFLARE_API_TOKEN
)

for name in "${required[@]}"; do
  if [[ -z "${!name:-}" ]]; then
    echo "missing required environment variable: $name" >&2
    exit 1
  fi
done

echo "== live Cloudflare KV portability proof =="
echo "Credentials are read from the environment and are not printed."

cargo run \
  -p presolve-node \
  --example cloudflare_kv_checkpoint

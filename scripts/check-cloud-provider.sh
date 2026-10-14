#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

echo "== provider realization tests =="
cargo test \
  -p presolve-provider-realization \
  -p presolve-provider-kv-cloudflare \
  -p presolve-provider-objects-s3 \
  -p presolve-node

echo "== provider realization clippy =="
cargo clippy \
  -p presolve-provider-realization \
  -p presolve-provider-kv-cloudflare \
  -p presolve-provider-objects-s3 \
  -p presolve-node \
  --all-targets \
  -- \
  -D warnings

echo "External-provider checkpoint passed"
echo "Live Cloudflare proof remains opt-in: ./scripts/smoke-cloudflare-kv.sh"
echo "S3-compatible object portability is exercised in: ./scripts/check-object-portability.sh"

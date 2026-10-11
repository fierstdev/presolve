#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

echo "== provider realization tests =="
cargo test \
  -p presolve-provider-realization \
  -p presolve-provider-kv-cloudflare \
  -p presolve-node

echo "== provider realization clippy =="
cargo clippy \
  -p presolve-provider-realization \
  -p presolve-provider-kv-cloudflare \
  -p presolve-node \
  --all-targets \
  -- \
  -D warnings

echo "Cloud-provider checkpoint passed"
echo "Live proof remains opt-in: ./scripts/smoke-cloudflare-kv.sh"

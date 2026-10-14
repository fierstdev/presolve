#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
APP="$ROOT/examples/kv-demo"
LOCAL_ENV="$ROOT/examples/portability-proof/local-memory/presolve.env.toml"
CLOUDFLARE_ENV="$ROOT/examples/portability-proof/managed-cloudflare/presolve.env.toml"
REDIS_ENV="$ROOT/examples/portability-proof/network-redis/presolve.env.toml"
PRESOLVE="$ROOT/target/debug/presolve"
TMP="$(mktemp -d "${TMPDIR:-/tmp}/presolve-release-realization.XXXXXX")"
RELEASE="$TMP/kv-demo.presolved"
RUN_ID="proof-$$"
LIVE_CLOUDFLARE=false
LIVE_REDIS=false
REDIS_CONTAINER=""

while [[ $# -gt 0 ]]; do
  case "$1" in
    --cloudflare)
      LIVE_CLOUDFLARE=true
      ;;
    --redis)
      LIVE_REDIS=true
      ;;
    *)
      echo "usage: $0 [--redis] [--cloudflare]" >&2
      exit 2
      ;;
  esac
  shift
done

cleanup() {
  if [[ -n "$REDIS_CONTAINER" ]]; then
    docker rm -f "$REDIS_CONTAINER" >/dev/null 2>&1 || true
  fi
  rm -rf "$TMP"
  rm -rf "$APP/.presolve" "$APP/dist"
}
trap cleanup EXIT

archive_sha256() {
  python3 - "$1" <<'PY'
from hashlib import sha256
from pathlib import Path
import sys

print(sha256(Path(sys.argv[1]).read_bytes()).hexdigest())
PY
}

required_cloudflare=(
  PRESOLVE_CLOUDFLARE_ACCOUNT_ID
  PRESOLVE_CLOUDFLARE_KV_NAMESPACE_ID
  PRESOLVE_CLOUDFLARE_API_TOKEN
)

if [[ "$LIVE_CLOUDFLARE" == true ]]; then
  for name in "${required_cloudflare[@]}"; do
    if [[ -z "${!name:-}" ]]; then
      echo "missing required environment variable: $name" >&2
      exit 1
    fi
  done
fi

start_redis() {
  docker version >/dev/null
  REDIS_CONTAINER="presolve-redis-proof-$$"

  docker run \
    --detach \
    --rm \
    --name "$REDIS_CONTAINER" \
    --publish 127.0.0.1::6379 \
    redis:7.4-alpine \
    redis-server --save "" --appendonly no >/dev/null

  local redis_address
  redis_address="$(docker port "$REDIS_CONTAINER" 6379/tcp | head -n 1)"
  if [[ -z "$redis_address" ]]; then
    echo "could not determine Redis proof address" >&2
    exit 1
  fi

  export PRESOLVE_REDIS_ADDRESS="$redis_address"

  for _ in $(seq 1 30); do
    if [[ "$(docker exec "$REDIS_CONTAINER" redis-cli ping 2>/dev/null || true)" == "PONG" ]]; then
      return
    fi
    sleep 1
  done

  echo "Redis proof container did not become ready" >&2
  docker logs "$REDIS_CONTAINER" >&2 || true
  exit 1
}

if [[ "$LIVE_REDIS" == true ]]; then
  start_redis
fi

echo "== build Presolve CLI =="
cargo build --manifest-path "$ROOT/Cargo.toml" -p presolve-cli

rm -rf "$APP/.presolve" "$APP/dist"

echo "== build immutable kv-demo release exactly once =="
(
  cd "$APP"
  "$PRESOLVE" build --output "$RELEASE"
)

test -f "$RELEASE"
ARCHIVE_SHA_BEFORE="$(archive_sha256 "$RELEASE")"
"$PRESOLVE" inspect "$RELEASE" --json > "$TMP/inspect.json"

RELEASE_DIGEST="$(python3 - "$TMP/inspect.json" <<'PY'
import json
from pathlib import Path
import sys

value = json.loads(Path(sys.argv[1]).read_text(encoding="utf-8"))
print(value["release"]["digest"])
PY
)"

echo "release digest: $RELEASE_DIGEST"
echo "archive sha256: $ARCHIVE_SHA_BEFORE"

echo "== execute exact release through local memory provider =="
cargo run \
  --manifest-path "$ROOT/Cargo.toml" \
  -p presolve-node \
  --example release_realization_checkpoint \
  -- \
  local \
  "$RELEASE" \
  "$LOCAL_ENV" \
  "$RUN_ID-local" | tee "$TMP/local.txt"

ARCHIVE_SHA_AFTER_LOCAL="$(archive_sha256 "$RELEASE")"
if [[ "$ARCHIVE_SHA_AFTER_LOCAL" != "$ARCHIVE_SHA_BEFORE" ]]; then
  echo "immutable release bytes changed after local execution" >&2
  exit 1
fi

grep -F "release digest: $RELEASE_DIGEST" "$TMP/local.txt" >/dev/null
grep -F "provider: memory-kv (prv_00000000-0000-4000-8000-000000000311)" "$TMP/local.txt" >/dev/null
grep -F "mode: local" "$TMP/local.txt" >/dev/null
grep -F "output: value:$RUN_ID-local" "$TMP/local.txt" >/dev/null

if [[ "$LIVE_REDIS" == true ]]; then
  echo "== execute the same exact release through network Redis =="

  cargo run \
    --manifest-path "$ROOT/Cargo.toml" \
    -p presolve-node \
    --example release_realization_checkpoint \
    -- \
    redis \
    "$RELEASE" \
    "$REDIS_ENV" \
    "$RUN_ID-redis" | tee "$TMP/redis.txt"

  ARCHIVE_SHA_AFTER_REDIS="$(archive_sha256 "$RELEASE")"
  if [[ "$ARCHIVE_SHA_AFTER_REDIS" != "$ARCHIVE_SHA_BEFORE" ]]; then
    echo "immutable release bytes changed after Redis execution" >&2
    exit 1
  fi

  grep -F "release digest: $RELEASE_DIGEST" "$TMP/redis.txt" >/dev/null
  grep -F "provider: redis-kv (prv_00000000-0000-4000-8000-000000000313)" "$TMP/redis.txt" >/dev/null
  grep -F "mode: redis" "$TMP/redis.txt" >/dev/null
  grep -F "output: value:$RUN_ID-redis" "$TMP/redis.txt" >/dev/null
fi

if [[ "$LIVE_CLOUDFLARE" == true ]]; then
  echo "== execute the same exact release through live Cloudflare KV =="
  echo "Cloudflare credentials are read from the environment and are not printed."

  cargo run \
    --manifest-path "$ROOT/Cargo.toml" \
    -p presolve-node \
    --example release_realization_checkpoint \
    -- \
    cloudflare \
    "$RELEASE" \
    "$CLOUDFLARE_ENV" \
    "$RUN_ID-cloudflare" | tee "$TMP/cloudflare.txt"

  ARCHIVE_SHA_AFTER_CLOUDFLARE="$(archive_sha256 "$RELEASE")"
  if [[ "$ARCHIVE_SHA_AFTER_CLOUDFLARE" != "$ARCHIVE_SHA_BEFORE" ]]; then
    echo "immutable release bytes changed after Cloudflare execution" >&2
    exit 1
  fi

  grep -F "release digest: $RELEASE_DIGEST" "$TMP/cloudflare.txt" >/dev/null
  grep -F "provider: cloudflare-kv (prv_00000000-0000-4000-8000-000000000312)" "$TMP/cloudflare.txt" >/dev/null
  grep -F "mode: cloudflare" "$TMP/cloudflare.txt" >/dev/null
  grep -F "output: value:$RUN_ID-cloudflare" "$TMP/cloudflare.txt" >/dev/null
fi

echo "archive sha256 unchanged: $ARCHIVE_SHA_BEFORE"
if [[ "$LIVE_REDIS" == true && "$LIVE_CLOUDFLARE" == true ]]; then
  echo "same-release local + Redis + Cloudflare realization proof passed"
elif [[ "$LIVE_REDIS" == true ]]; then
  echo "same-release local + Redis realization proof passed"
elif [[ "$LIVE_CLOUDFLARE" == true ]]; then
  echo "same-release local + Cloudflare realization proof passed"
else
  echo "detached-release local realization checkpoint passed"
fi

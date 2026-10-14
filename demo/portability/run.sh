#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
APP="$ROOT/examples/kv-demo"
LOCAL_ENV="$ROOT/examples/portability-proof/local-memory"
REDIS_ENV="$ROOT/examples/portability-proof/network-redis"
CLOUDFLARE_ENV="$ROOT/examples/portability-proof/managed-cloudflare"
PRESOLVE="$ROOT/target/debug/presolve"
CHECKPOINT="$ROOT/target/debug/examples/release_realization_checkpoint"
TMP="$(mktemp -d "${TMPDIR:-/tmp}/presolve-flagship-demo.XXXXXX")"
RELEASE="$TMP/kv-demo.presolved"
RUN_ID="demo-$$"
REDIS_CONTAINER=""
LIVE_CLOUDFLARE=false

usage() {
  cat <<'EOF'
Usage: bash ./demo/portability/run.sh [--cloudflare]

Runs Presolve's flagship portability demonstration:
  - builds one immutable .presolved application release exactly once;
  - resolves those exact bytes against memory, Redis, and Cloudflare environments;
  - executes those exact bytes through memory and a real Redis process;
  - optionally executes the same release through live Cloudflare Workers KV; and
  - verifies the release digest and archive SHA-256 remain unchanged.

--cloudflare  Also execute against live Cloudflare Workers KV. Requires:
              PRESOLVE_CLOUDFLARE_ACCOUNT_ID
              PRESOLVE_CLOUDFLARE_KV_NAMESPACE_ID
              PRESOLVE_CLOUDFLARE_API_TOKEN
EOF
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --cloudflare)
      LIVE_CLOUDFLARE=true
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      usage >&2
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

require_command() {
  if ! command -v "$1" >/dev/null 2>&1; then
    echo "missing required command: $1" >&2
    exit 1
  fi
}

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
  REDIS_CONTAINER="presolve-flagship-redis-$$"

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
    echo "could not determine Redis demo address" >&2
    exit 1
  fi

  export PRESOLVE_REDIS_ADDRESS="$redis_address"

  for _ in $(seq 1 30); do
    if [[ "$(docker exec "$REDIS_CONTAINER" redis-cli ping 2>/dev/null || true)" == "PONG" ]]; then
      return
    fi
    sleep 1
  done

  echo "Redis demo container did not become ready" >&2
  docker logs "$REDIS_CONTAINER" >&2 || true
  exit 1
}

plan_release() {
  local environment_dir="$1"
  local output="$2"

  (
    cd "$environment_dir"
    "$PRESOLVE" plan "$RELEASE" --json > "$output"
  )
}

execute_release() {
  local mode="$1"
  local environment_file="$2"
  local input="$3"
  local output="$4"

  if ! "$CHECKPOINT" \
    "$mode" \
    "$RELEASE" \
    "$environment_file" \
    "$input" > "$output" 2>&1; then
    cat "$output" >&2
    exit 1
  fi
}

require_command cargo
require_command docker
require_command python3

echo "Presolve flagship portability demonstration"
echo "=========================================="
echo "Claim: build one immutable application release once; decide where it runs later."
echo

echo "[1/5] Build Presolve tooling"
cargo build --quiet --manifest-path "$ROOT/Cargo.toml" -p presolve-cli
cargo build --quiet --manifest-path "$ROOT/Cargo.toml" -p presolve-node --example release_realization_checkpoint

echo "[2/5] Build one immutable application release"
rm -rf "$APP/.presolve" "$APP/dist"
(
  cd "$APP"
  "$PRESOLVE" build --output "$RELEASE" >/dev/null
)

test -f "$RELEASE"
ARCHIVE_SHA_BEFORE="$(archive_sha256 "$RELEASE")"
"$PRESOLVE" inspect "$RELEASE" --json > "$TMP/inspect-before.json"

RELEASE_DIGEST="$(python3 - "$TMP/inspect-before.json" <<'PY'
import json
from pathlib import Path
import sys

value = json.loads(Path(sys.argv[1]).read_text(encoding="utf-8"))
assert value["kind"] == "release_inspection"
assert value["application"]["name"] == "kv-demo"
print(value["release"]["digest"])
PY
)"

printf "  release digest: %s\n" "$RELEASE_DIGEST"
printf "  archive sha256: %s\n" "$ARCHIVE_SHA_BEFORE"
echo

echo "[3/5] Resolve the same release against three environments"
plan_release "$LOCAL_ENV" "$TMP/local-plan.json"
plan_release "$REDIS_ENV" "$TMP/redis-plan.json"
plan_release "$CLOUDFLARE_ENV" "$TMP/cloudflare-plan.json"

python3 - \
  "$TMP/local-plan.json" \
  "$TMP/redis-plan.json" \
  "$TMP/cloudflare-plan.json" \
  "$RELEASE" \
  "$RELEASE_DIGEST" <<'PY'
import json
from pathlib import Path
import sys

local_path, redis_path, cloudflare_path, release_path, expected_digest = sys.argv[1:]
release = str(Path(release_path).resolve())


def load(path: str) -> dict:
    return json.loads(Path(path).read_text(encoding="utf-8"))


rows = []
expected = [
    (load(local_path), "portability-local", "memory-kv"),
    (load(redis_path), "portability-redis", "redis-kv"),
    (load(cloudflare_path), "portability-cloudflare", "cloudflare-kv"),
]

for plan, environment_name, provider_name in expected:
    assert plan["kind"] == "deployment_plan"
    assert plan["status"] == "resolved"
    assert plan["problems"] == []
    assert plan["application"]["release"] == release
    assert plan["application"]["release_digest"] == expected_digest
    assert plan["environment"]["name"] == environment_name
    binding = plan["plan"]["bindings"][0]
    assert binding["interface"] == "presolve:kv/store"
    assert binding["provider_name"] == provider_name
    rows.append(
        (
            environment_name,
            provider_name,
            plan["plan"]["environment_id"],
            binding["provider_id"],
        )
    )

assert len({row[2] for row in rows}) == 3
assert len({row[3] for row in rows}) == 3

print(f"  {'environment':24} {'selected provider':18} {'plan'}")
print(f"  {'-' * 24} {'-' * 18} {'--------'}")
for environment, provider, _, _ in rows:
    print(f"  {environment:24} {provider:18} resolved")
PY

echo

echo "[4/5] Realize and execute those exact release bytes"
start_redis

execute_release \
  local \
  "$LOCAL_ENV/presolve.env.toml" \
  "$RUN_ID-local" \
  "$TMP/local-execution.txt"

execute_release \
  redis \
  "$REDIS_ENV/presolve.env.toml" \
  "$RUN_ID-redis" \
  "$TMP/redis-execution.txt"

if [[ "$LIVE_CLOUDFLARE" == true ]]; then
  execute_release \
    cloudflare \
    "$CLOUDFLARE_ENV/presolve.env.toml" \
    "$RUN_ID-cloudflare" \
    "$TMP/cloudflare-execution.txt"
fi

python3 - \
  "$TMP/local-execution.txt" \
  "$TMP/redis-execution.txt" \
  "$TMP/cloudflare-execution.txt" \
  "$LIVE_CLOUDFLARE" \
  "$RELEASE_DIGEST" <<'PY'
from pathlib import Path
import sys

local_path, redis_path, cloudflare_path, live_cloudflare, expected_digest = sys.argv[1:]


def parse(path: str) -> dict[str, str]:
    values = {}
    for line in Path(path).read_text(encoding="utf-8").splitlines():
        if ": " in line:
            key, value = line.split(": ", 1)
            values[key] = value
    return values


rows = []
for path, environment, provider in (
    (local_path, "portability-local", "memory-kv"),
    (redis_path, "portability-redis", "redis-kv"),
):
    values = parse(path)
    assert values["release digest"] == expected_digest
    assert values["environment"].startswith(f"{environment} (")
    assert values["provider"].startswith(f"{provider} (")
    assert values["output"].startswith("value:demo-")
    rows.append((environment, provider, values["output"], "executed"))

if live_cloudflare == "true":
    values = parse(cloudflare_path)
    assert values["release digest"] == expected_digest
    assert values["environment"].startswith("portability-cloudflare (")
    assert values["provider"].startswith("cloudflare-kv (")
    assert values["output"].startswith("value:demo-")
    rows.append(("portability-cloudflare", "cloudflare-kv", values["output"], "executed"))
else:
    rows.append(("portability-cloudflare", "cloudflare-kv", "--cloudflare", "plan only"))

print(f"  {'environment':24} {'realized provider':18} {'result':24} status")
print(f"  {'-' * 24} {'-' * 18} {'-' * 24} {'--------'}")
for environment, provider, result, status in rows:
    print(f"  {environment:24} {provider:18} {result:24} {status}")
PY

echo

echo "[5/5] Verify application identity never changed"
ARCHIVE_SHA_AFTER="$(archive_sha256 "$RELEASE")"
"$PRESOLVE" inspect "$RELEASE" --json > "$TMP/inspect-after.json"

FINAL_RELEASE_DIGEST="$(python3 - "$TMP/inspect-after.json" <<'PY'
import json
from pathlib import Path
import sys

value = json.loads(Path(sys.argv[1]).read_text(encoding="utf-8"))
print(value["release"]["digest"])
PY
)"

if [[ "$FINAL_RELEASE_DIGEST" != "$RELEASE_DIGEST" ]]; then
  echo "release digest changed during the demonstration" >&2
  exit 1
fi

if [[ "$ARCHIVE_SHA_AFTER" != "$ARCHIVE_SHA_BEFORE" ]]; then
  echo "encoded .presolved bytes changed during the demonstration" >&2
  exit 1
fi

printf "  release digest unchanged: yes (%s)\n" "$RELEASE_DIGEST"
printf "  archive sha256 unchanged: yes (%s)\n" "$ARCHIVE_SHA_BEFORE"
echo

if [[ "$LIVE_CLOUDFLARE" == true ]]; then
  echo "PASS — one .presolved release, three plans, three provider realizations, zero application rebuilds."
else
  echo "PASS — one .presolved release, three plans, two provider realizations, zero application rebuilds."
  echo "       Re-run with --cloudflare to execute the same release through live Cloudflare Workers KV."
fi

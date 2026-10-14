#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
APP="$ROOT/examples/objects-demo"
LOCAL_ENV="$ROOT/examples/portability-proof/objects-local"
S3_ENV="$ROOT/examples/portability-proof/objects-s3"
PRESOLVE="$ROOT/target/debug/presolve"
CHECKPOINT="$ROOT/target/debug/examples/object_release_realization_checkpoint"
TMP="$(mktemp -d "${TMPDIR:-/tmp}/presolve-object-portability.XXXXXX")"
RELEASE="$TMP/objects-demo.presolved"
RUN_ID="objects-$$"
MINIO_CONTAINER="presolve-object-minio-$$"
MINIO_IMAGE="ghcr.io/coollabsio/minio:RELEASE.2025-10-15T17-29-55Z@sha256:69b55a1c1c5dc285ce04db96689f5b2102317fc77a50680a1874ca6efd1c87f9"

export PRESOLVE_S3_BUCKET="presolve-object-proof"
export PRESOLVE_S3_REGION="us-east-1"
export PRESOLVE_S3_ACCESS_KEY_ID="presolve-access"
export PRESOLVE_S3_SECRET_ACCESS_KEY="presolve-secret-key-123"

cleanup() {
  docker rm -f "$MINIO_CONTAINER" >/dev/null 2>&1 || true
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

start_s3() {
  docker run \
    --detach \
    --rm \
    --name "$MINIO_CONTAINER" \
    --env "MINIO_ROOT_USER=$PRESOLVE_S3_ACCESS_KEY_ID" \
    --env "MINIO_ROOT_PASSWORD=$PRESOLVE_S3_SECRET_ACCESS_KEY" \
    --publish 127.0.0.1::9000 \
    "$MINIO_IMAGE" \
    server /data --address :9000 >/dev/null

  local address
  address="$(docker port "$MINIO_CONTAINER" 9000/tcp | head -n 1)"
  if [[ -z "$address" ]]; then
    echo "could not determine S3-compatible test endpoint" >&2
    exit 1
  fi

  export PRESOLVE_S3_ENDPOINT="http://$address"

  for _ in $(seq 1 60); do
    if python3 - "$PRESOLVE_S3_ENDPOINT" <<'PY' >/dev/null 2>&1
import sys
import urllib.request

with urllib.request.urlopen(f"{sys.argv[1]}/minio/health/ready", timeout=1) as response:
    if response.status != 200:
        raise SystemExit(1)
PY
    then
      create_bucket
      return
    fi
    sleep 1
  done

  echo "S3-compatible test server did not become ready" >&2
  docker logs "$MINIO_CONTAINER" >&2 || true
  exit 1
}

create_bucket() {
  python3 - <<'PY'
from datetime import datetime, timezone
from hashlib import sha256
import hmac
import os
from urllib.parse import quote, urlparse
from urllib.request import Request, urlopen

endpoint = os.environ["PRESOLVE_S3_ENDPOINT"].rstrip("/")
bucket = os.environ["PRESOLVE_S3_BUCKET"]
region = os.environ["PRESOLVE_S3_REGION"]
access_key = os.environ["PRESOLVE_S3_ACCESS_KEY_ID"]
secret_key = os.environ["PRESOLVE_S3_SECRET_ACCESS_KEY"]
parsed = urlparse(endpoint)
host = parsed.netloc
path = "/" + quote(bucket, safe="-_.~")
now = datetime.now(timezone.utc)
amz_date = now.strftime("%Y%m%dT%H%M%SZ")
date = now.strftime("%Y%m%d")
payload_hash = sha256(b"").hexdigest()
canonical_headers = (
    f"host:{host}\n"
    f"x-amz-content-sha256:{payload_hash}\n"
    f"x-amz-date:{amz_date}\n"
)
signed_headers = "host;x-amz-content-sha256;x-amz-date"
canonical_request = (
    f"PUT\n{path}\n\n{canonical_headers}\n{signed_headers}\n{payload_hash}"
)
scope = f"{date}/{region}/s3/aws4_request"
string_to_sign = (
    f"AWS4-HMAC-SHA256\n{amz_date}\n{scope}\n"
    f"{sha256(canonical_request.encode()).hexdigest()}"
)


def sign(key: bytes, value: str) -> bytes:
    return hmac.new(key, value.encode(), sha256).digest()


key = sign(("AWS4" + secret_key).encode(), date)
key = sign(key, region)
key = sign(key, "s3")
key = sign(key, "aws4_request")
signature = hmac.new(key, string_to_sign.encode(), sha256).hexdigest()
authorization = (
    f"AWS4-HMAC-SHA256 Credential={access_key}/{scope}, "
    f"SignedHeaders={signed_headers}, Signature={signature}"
)
request = Request(
    endpoint + path,
    method="PUT",
    headers={
        "Authorization": authorization,
        "x-amz-content-sha256": payload_hash,
        "x-amz-date": amz_date,
    },
)
with urlopen(request, timeout=5) as response:
    if response.status not in (200, 204):
        raise SystemExit(f"unexpected create-bucket status: {response.status}")
PY
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

echo "Presolve object-storage portability checkpoint"
echo "=============================================="
echo "Claim: one immutable object-capable release can resolve and execute through materially different object providers."
echo

echo "[1/5] Build Presolve tooling"
cargo build --quiet --manifest-path "$ROOT/Cargo.toml" -p presolve-cli
cargo build --quiet --manifest-path "$ROOT/Cargo.toml" -p presolve-node --example object_release_realization_checkpoint

echo "[2/5] Build one immutable object application release"
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
assert value["application"]["name"] == "objects-demo"
print(value["release"]["digest"])
PY
)"

printf "  release digest: %s\n" "$RELEASE_DIGEST"
printf "  archive sha256: %s\n" "$ARCHIVE_SHA_BEFORE"
echo

echo "[3/5] Resolve the same release against memory and S3-compatible environments"
plan_release "$LOCAL_ENV" "$TMP/local-plan.json"
plan_release "$S3_ENV" "$TMP/s3-plan.json"

python3 - "$TMP/local-plan.json" "$TMP/s3-plan.json" "$RELEASE" "$RELEASE_DIGEST" <<'PY'
import json
from pathlib import Path
import sys

local_path, s3_path, release_path, expected_digest = sys.argv[1:]
release = str(Path(release_path).resolve())
expected = [
    (json.loads(Path(local_path).read_text()), "objects-local", "memory-objects"),
    (json.loads(Path(s3_path).read_text()), "objects-s3", "s3-objects"),
]
rows = []

for plan, environment_name, provider_name in expected:
    assert plan["kind"] == "deployment_plan"
    assert plan["status"] == "resolved"
    assert plan["problems"] == []
    assert plan["application"]["release"] == release
    assert plan["application"]["release_digest"] == expected_digest
    assert plan["environment"]["name"] == environment_name
    binding = plan["plan"]["bindings"][0]
    assert binding["interface"] == "presolve:objects/store"
    assert binding["provider_name"] == provider_name
    rows.append((environment_name, provider_name, binding["provider_id"]))

assert rows[0][2] != rows[1][2]
print(f"  {'environment':20} {'selected provider':18} plan")
print(f"  {'-' * 20} {'-' * 18} {'--------'}")
for environment, provider, _ in rows:
    print(f"  {environment:20} {provider:18} resolved")
PY

echo

echo "[4/5] Execute those exact release bytes through both providers"
start_s3
execute_release \
  local \
  "$LOCAL_ENV/presolve.env.toml" \
  "$RUN_ID-local" \
  "$TMP/local-execution.txt"
execute_release \
  s3 \
  "$S3_ENV/presolve.env.toml" \
  "$RUN_ID-s3" \
  "$TMP/s3-execution.txt"

python3 - "$TMP/local-execution.txt" "$TMP/s3-execution.txt" "$RELEASE_DIGEST" <<'PY'
from pathlib import Path
import sys

local_path, s3_path, expected_digest = sys.argv[1:]
expected = [
    (local_path, "objects-local", "memory-objects"),
    (s3_path, "objects-s3", "s3-objects"),
]

print(f"  {'environment':20} {'realized provider':18} result")
print(f"  {'-' * 20} {'-' * 18} {'----------------'}")
for path, environment, provider in expected:
    values = {}
    for line in Path(path).read_text(encoding="utf-8").splitlines():
        if ": " in line:
            key, value = line.split(": ", 1)
            values[key] = value
    assert values["release digest"] == expected_digest
    assert values["environment"].startswith(f"{environment} (")
    assert values["provider"].startswith(f"{provider} (")
    assert values["output"].startswith("objects:objects-")
    print(f"  {environment:20} {provider:18} {values['output']}")
PY

echo

echo "[5/5] Verify application identity never changed"
ARCHIVE_SHA_AFTER="$(archive_sha256 "$RELEASE")"
"$PRESOLVE" inspect "$RELEASE" --json > "$TMP/inspect-after.json"
FINAL_RELEASE_DIGEST="$(python3 - "$TMP/inspect-after.json" <<'PY'
import json
from pathlib import Path
import sys

print(json.loads(Path(sys.argv[1]).read_text())["release"]["digest"])
PY
)"

if [[ "$FINAL_RELEASE_DIGEST" != "$RELEASE_DIGEST" ]]; then
  echo "release digest changed during object portability checkpoint" >&2
  exit 1
fi

if [[ "$ARCHIVE_SHA_AFTER" != "$ARCHIVE_SHA_BEFORE" ]]; then
  echo "encoded .presolved bytes changed during object portability checkpoint" >&2
  exit 1
fi

printf "  release digest unchanged: yes (%s)\n" "$RELEASE_DIGEST"
printf "  archive sha256 unchanged: yes (%s)\n" "$ARCHIVE_SHA_BEFORE"
echo
echo "PASS — one .presolved object release, two plans, two provider realizations, zero application rebuilds."

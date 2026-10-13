#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
APP="$ROOT/examples/kv-demo"
LOCAL_ENV="$ROOT/examples/portability-proof/local-memory"
CLOUDFLARE_ENV="$ROOT/examples/portability-proof/managed-cloudflare"
PRESOLVE="$ROOT/target/debug/presolve"
TMP="$(mktemp -d "${TMPDIR:-/tmp}/presolve-portability-proof.XXXXXX")"
RELEASE="$TMP/kv-demo.presolved"

cleanup() {
  rm -rf "$TMP"
  rm -rf "$APP/.presolve" "$APP/dist"
}
trap cleanup EXIT

archive_sha256() {
  python3 - "$1" <<'PY'
from hashlib import sha256
from pathlib import Path
import sys

path = Path(sys.argv[1])
print(sha256(path.read_bytes()).hexdigest())
PY
}

echo "== build Presolve CLI =="
cargo build --manifest-path "$ROOT/Cargo.toml" -p presolve-cli

rm -rf "$APP/.presolve" "$APP/dist"

echo "== build kv-demo immutable release exactly once =="
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
assert value["kind"] == "release_inspection"
assert value["application"]["name"] == "kv-demo"
digest = value["release"]["digest"]
assert digest.startswith("sha256:")
print(digest)
PY
)"

echo "release file:   $RELEASE"
echo "release digest: $RELEASE_DIGEST"
echo "archive sha256: $ARCHIVE_SHA_BEFORE"

echo "== resolve exact release in local-memory environment =="
(
  cd "$LOCAL_ENV"
  "$PRESOLVE" check "$RELEASE" --json > "$TMP/local-check.json"
  "$PRESOLVE" plan "$RELEASE" --json > "$TMP/local-plan.json"
)

ARCHIVE_SHA_AFTER_LOCAL="$(archive_sha256 "$RELEASE")"
if [[ "$ARCHIVE_SHA_AFTER_LOCAL" != "$ARCHIVE_SHA_BEFORE" ]]; then
  echo "immutable release bytes changed after local resolution" >&2
  exit 1
fi

echo "== resolve exact release in managed-cloudflare environment =="
(
  cd "$CLOUDFLARE_ENV"
  "$PRESOLVE" check "$RELEASE" --json > "$TMP/cloudflare-check.json"
  "$PRESOLVE" plan "$RELEASE" --json > "$TMP/cloudflare-plan.json"
)

ARCHIVE_SHA_AFTER_CLOUDFLARE="$(archive_sha256 "$RELEASE")"
if [[ "$ARCHIVE_SHA_AFTER_CLOUDFLARE" != "$ARCHIVE_SHA_BEFORE" ]]; then
  echo "immutable release bytes changed after managed resolution" >&2
  exit 1
fi

echo "== verify one release produced two environment-specific plans =="
python3 - \
  "$TMP/inspect.json" \
  "$TMP/local-check.json" \
  "$TMP/local-plan.json" \
  "$TMP/cloudflare-check.json" \
  "$TMP/cloudflare-plan.json" \
  "$RELEASE" \
  "$RELEASE_DIGEST" <<'PY'
import json
from pathlib import Path
import sys

(
    inspect_path,
    local_check_path,
    local_plan_path,
    cloudflare_check_path,
    cloudflare_plan_path,
    release_path,
    expected_digest,
) = sys.argv[1:]


def load(path: str) -> dict:
    return json.loads(Path(path).read_text(encoding="utf-8"))


inspect = load(inspect_path)
local_check = load(local_check_path)
local_plan = load(local_plan_path)
cloudflare_check = load(cloudflare_check_path)
cloudflare_plan = load(cloudflare_plan_path)
release = str(Path(release_path).resolve())

assert inspect["release"]["digest"] == expected_digest

for check in (local_check, cloudflare_check):
    assert check["kind"] == "compatibility_check"
    assert check["status"] == "compatible"
    assert check["problems"] == []
    assert check["application"]["name"] == "kv-demo"
    assert check["application"]["contract"] is None
    assert check["application"]["release"] == release
    assert check["application"]["release_digest"] == expected_digest

for plan in (local_plan, cloudflare_plan):
    assert plan["kind"] == "deployment_plan"
    assert plan["status"] == "resolved"
    assert plan["problems"] == []
    assert plan["application"]["name"] == "kv-demo"
    assert plan["application"]["contract"] is None
    assert plan["application"]["release"] == release
    assert plan["application"]["release_digest"] == expected_digest
    assert plan["plan"] is not None
    assert len(plan["plan"]["bindings"]) == 1
    assert plan["plan"]["bindings"][0]["interface"] == "presolve:kv/store"

assert local_check["environment"]["id"] == "env_00000000-0000-4000-8000-000000000301"
assert local_check["environment"]["name"] == "portability-local"
assert local_plan["plan"]["environment_id"] == "env_00000000-0000-4000-8000-000000000301"
assert local_plan["plan"]["bindings"][0]["provider_id"] == "prv_00000000-0000-4000-8000-000000000311"
assert local_plan["plan"]["bindings"][0]["provider_name"] == "memory-kv"

assert cloudflare_check["environment"]["id"] == "env_00000000-0000-4000-8000-000000000302"
assert cloudflare_check["environment"]["name"] == "portability-cloudflare"
assert cloudflare_plan["plan"]["environment_id"] == "env_00000000-0000-4000-8000-000000000302"
assert cloudflare_plan["plan"]["bindings"][0]["provider_id"] == "prv_00000000-0000-4000-8000-000000000312"
assert cloudflare_plan["plan"]["bindings"][0]["provider_name"] == "cloudflare-kv"

assert local_plan["plan"]["environment_id"] != cloudflare_plan["plan"]["environment_id"]
assert (
    local_plan["plan"]["bindings"][0]["provider_id"]
    != cloudflare_plan["plan"]["bindings"][0]["provider_id"]
)

print(f"immutable release: {expected_digest}")
print(
    "local plan:       "
    f"{local_plan['plan']['environment_id']} -> "
    f"{local_plan['plan']['bindings'][0]['provider_name']}"
)
print(
    "managed plan:     "
    f"{cloudflare_plan['plan']['environment_id']} -> "
    f"{cloudflare_plan['plan']['bindings'][0]['provider_name']}"
)
PY

echo "archive sha256 unchanged: $ARCHIVE_SHA_BEFORE"
echo "portability proof checkpoint passed"

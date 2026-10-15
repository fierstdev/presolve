#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
APP="$ROOT/examples/queue-demo"
QUEUE_ENV="$ROOT/examples/portability-proof/queue-memory"
PRESOLVE="$ROOT/target/debug/presolve"
CHECKPOINT="$ROOT/target/debug/examples/queue_release_realization_checkpoint"
TMP="$(mktemp -d "${TMPDIR:-/tmp}/presolve-queue-realization.XXXXXX")"
RELEASE="$TMP/queue-demo.presolved"
RUN_ID="queue-$$"

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

print(sha256(Path(sys.argv[1]).read_bytes()).hexdigest())
PY
}

plan_release() {
  (
    cd "$QUEUE_ENV"
    "$PRESOLVE" plan "$RELEASE" --json > "$TMP/plan.json"
  )
}

echo "Presolve work-queue realization checkpoint"
echo "=========================================="
echo "Claim: a detached immutable queue-capable release can be planned, realized, and executed without rebuilding the application."
echo

echo "[1/5] Build Presolve tooling"
cargo build --quiet --manifest-path "$ROOT/Cargo.toml" -p presolve-cli
cargo build --quiet --manifest-path "$ROOT/Cargo.toml" -p presolve-node --example queue_release_realization_checkpoint

echo "[2/5] Build one immutable queue application release"
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
assert value["application"]["name"] == "queue-demo"
print(value["release"]["digest"])
PY
)"

printf "  release digest: %s\n" "$RELEASE_DIGEST"
printf "  archive sha256: %s\n" "$ARCHIVE_SHA_BEFORE"
echo

echo "[3/5] Resolve the detached release against the memory queue environment"
plan_release

python3 - "$TMP/plan.json" "$RELEASE" "$RELEASE_DIGEST" <<'PY'
import json
from pathlib import Path
import sys

plan_path, release_path, expected_digest = sys.argv[1:]
plan = json.loads(Path(plan_path).read_text(encoding="utf-8"))
release = str(Path(release_path).resolve())

assert plan["kind"] == "deployment_plan"
assert plan["status"] == "resolved"
assert plan["problems"] == []
assert plan["application"]["release"] == release
assert plan["application"]["release_digest"] == expected_digest
assert plan["environment"]["name"] == "queue-memory"
binding = plan["plan"]["bindings"][0]
assert binding["interface"] == "presolve:queue/work-queue"
assert binding["provider_name"] == "memory-queue"

print(f"  environment: {plan['environment']['name']}")
print(f"  selected provider: {binding['provider_name']} ({binding['provider_id']})")
print("  plan: resolved")
PY

echo

echo "[4/5] Execute those exact release bytes through the realized queue provider"
if ! "$CHECKPOINT" \
  "$RELEASE" \
  "$QUEUE_ENV/presolve.env.toml" \
  "$RUN_ID" > "$TMP/execution.txt" 2>&1; then
  cat "$TMP/execution.txt" >&2
  exit 1
fi

python3 - "$TMP/execution.txt" "$RELEASE_DIGEST" "$RUN_ID" <<'PY'
from pathlib import Path
import sys

path, expected_digest, run_id = sys.argv[1:]
values = {}
for line in Path(path).read_text(encoding="utf-8").splitlines():
    if ": " in line:
        key, value = line.split(": ", 1)
        values[key] = value

assert values["release digest"] == expected_digest
assert values["environment"].startswith("queue-memory (")
assert values["provider"].startswith("memory-queue (")
assert values["output"] == f"queue:{run_id}"

print(f"  environment: {values['environment']}")
print(f"  realized provider: {values['provider']}")
print(f"  result: {values['output']}")
PY

echo

echo "[5/5] Verify application identity never changed"
ARCHIVE_SHA_AFTER="$(archive_sha256 "$RELEASE")"
"$PRESOLVE" inspect "$RELEASE" --json > "$TMP/inspect-after.json"
FINAL_RELEASE_DIGEST="$(python3 - "$TMP/inspect-after.json" <<'PY'
import json
from pathlib import Path
import sys

print(json.loads(Path(sys.argv[1]).read_text(encoding="utf-8"))["release"]["digest"])
PY
)"

if [[ "$FINAL_RELEASE_DIGEST" != "$RELEASE_DIGEST" ]]; then
  echo "release digest changed during queue realization checkpoint" >&2
  exit 1
fi

if [[ "$ARCHIVE_SHA_AFTER" != "$ARCHIVE_SHA_BEFORE" ]]; then
  echo "encoded .presolved bytes changed during queue realization checkpoint" >&2
  exit 1
fi

printf "  release digest unchanged: yes (%s)\n" "$RELEASE_DIGEST"
printf "  archive sha256 unchanged: yes (%s)\n" "$ARCHIVE_SHA_BEFORE"
echo
echo "PASS — one .presolved queue release, one plan, one provider realization, zero application rebuilds."

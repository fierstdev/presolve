#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
PRESOLVE="$ROOT/target/debug/presolve"
TMP="$(mktemp -d "${TMPDIR:-/tmp}/presolve-inspection-planning.XXXXXX")"

cleanup() {
  rm -rf "$TMP"
}
trap cleanup EXIT

echo "== inspection/planning Rust tests =="
cargo test --manifest-path "$ROOT/Cargo.toml" \
  -p presolve-bundle \
  -p presolve-cli

echo "== inspection/planning Clippy =="
cargo clippy --manifest-path "$ROOT/Cargo.toml" \
  -p presolve-bundle \
  -p presolve-cli \
  --all-targets \
  -- \
  -D warnings

echo "== build Presolve CLI =="
cargo build --manifest-path "$ROOT/Cargo.toml" -p presolve-cli

RESOLVED="$TMP/resolved"
REJECTED="$TMP/rejected"
DETACHED="$TMP/detached"

mkdir -p "$RESOLVED" "$REJECTED" "$DETACHED"

cat > "$RESOLVED/presolve.toml" <<'EOF'
contract_version = "0.1"

[application]
name = "ux-checkpoint"
version = "1.0.0"
description = "Chunk 20 UX checkpoint"

[[capabilities]]
interface = "presolve:kv/store"
version = "^0.1"

[resources]
memory_mib = 64
cpu_millis = 100

[network]
outbound = "deny"
EOF

printf 'chunk 20 component bytes\n' > "$RESOLVED/app.wasm"

echo "== source inspection without environment =="
(
  cd "$RESOLVED"

  "$PRESOLVE" inspect > "$TMP/inspect-human.txt"
  "$PRESOLVE" inspect --json > "$TMP/inspect.json"
)

grep -q 'application: ux-checkpoint 1.0.0' "$TMP/inspect-human.txt"
grep -q 'main -> component (implicit)' "$TMP/inspect-human.txt"

python3 - "$TMP/inspect.json" <<'PY'
import json
import sys

with open(sys.argv[1], encoding="utf-8") as handle:
    value = json.load(handle)

assert value["schema"] == "presolve.cli"
assert value["schema_version"] == "0.1"
assert value["kind"] == "application_inspection"
assert value["application"]["name"] == "ux-checkpoint"
assert value["workloads"] == [
    {"name": "main", "kind": "component", "implicit": True}
]
assert value["capabilities"][0]["interface"] == "presolve:kv/store"
PY

echo "== resolved environment =="
(
  cd "$RESOLVED"

  "$PRESOLVE" env init local > /dev/null
  "$PRESOLVE" env provider add memory-services --provides kv > /dev/null
  "$PRESOLVE" env check > /dev/null

  "$PRESOLVE" check > "$TMP/check-human.txt"
  "$PRESOLVE" check --json > "$TMP/check.json"

  "$PRESOLVE" plan > "$TMP/plan-human.txt"
  "$PRESOLVE" plan --json > "$TMP/plan.json"
)

grep -q 'Status: compatible' "$TMP/check-human.txt"
grep -q 'presolve:kv/store -> memory-services' "$TMP/check-human.txt"

grep -q 'Presolve plan' "$TMP/plan-human.txt"
grep -q 'provider: memory-services' "$TMP/plan-human.txt"
grep -q 'semantic contract: presolve:kv/store@0.1.0' "$TMP/plan-human.txt"

python3 - "$TMP/check.json" "$TMP/plan.json" <<'PY'
import json
import sys

with open(sys.argv[1], encoding="utf-8") as handle:
    check = json.load(handle)

with open(sys.argv[2], encoding="utf-8") as handle:
    plan = json.load(handle)

assert check["schema"] == "presolve.cli"
assert check["schema_version"] == "0.1"
assert check["kind"] == "compatibility_check"
assert check["status"] == "compatible"
assert check["problems"] == []

assert plan["schema"] == "presolve.cli"
assert plan["schema_version"] == "0.1"
assert plan["kind"] == "deployment_plan"
assert plan["status"] == "resolved"
assert plan["plan"] is not None
assert plan["problems"] == []
assert plan["plan"]["workloads"] == [{"name": "main", "kind": "component"}]

binding = plan["plan"]["bindings"][0]
assert binding["interface"] == "presolve:kv/store"
assert binding["provider_name"] == "memory-services"
assert binding["provider_version"] == "0.1.0"
assert binding["provider_id"].startswith("prv_")
assert binding["semantic_contract"]["interface"] == "presolve:kv/store"
assert binding["semantic_contract"]["version"] == "0.1.0"
PY

echo "== immutable release inspection detached from source/environment =="
(
  cd "$RESOLVED"
  "$PRESOLVE" build \
    --component app.wasm \
    --output ux-checkpoint.presolved > /dev/null
)

cp "$RESOLVED/ux-checkpoint.presolved" "$DETACHED/ux-checkpoint.presolved"

(
  cd "$DETACHED"
  "$PRESOLVE" inspect ux-checkpoint.presolved > "$TMP/release-human.txt"
  "$PRESOLVE" inspect ux-checkpoint.presolved --json > "$TMP/release.json"
)

grep -q 'Presolve release' "$TMP/release-human.txt"
grep -q 'bundle version: 0.2' "$TMP/release-human.txt"
grep -q 'release digest: sha256:' "$TMP/release-human.txt"
grep -q 'artifact: artifacts/main.wasm' "$TMP/release-human.txt"

python3 - "$TMP/release.json" <<'PY'
import json
import sys

with open(sys.argv[1], encoding="utf-8") as handle:
    value = json.load(handle)

assert value["schema"] == "presolve.cli"
assert value["schema_version"] == "0.1"
assert value["kind"] == "release_inspection"
assert value["release"]["bundle_version"] == "0.2"
assert value["release"]["digest"].startswith("sha256:")
assert value["application"]["name"] == "ux-checkpoint"

workload = value["workloads"][0]
assert workload["name"] == "main"
assert workload["kind"] == "component"
assert workload["artifacts"][0]["path"] == "artifacts/main.wasm"
assert workload["artifacts"][0]["digest"].startswith("sha256:")
assert workload["oci_image"] is None
PY

cat > "$REJECTED/presolve.toml" <<'EOF'
contract_version = "0.1"

[application]
name = "ux-rejected"
version = "1.0.0"

[[capabilities]]
interface = "presolve:kv/store"
version = "^0.1"
EOF

(
  cd "$REJECTED"
  "$PRESOLVE" env init local > /dev/null
)

echo "== rejected compatibility JSON remains machine-readable =="
set +e
(
  cd "$REJECTED"
  "$PRESOLVE" check --json
) > "$TMP/rejected-check.json" 2> "$TMP/rejected-check.err"
CHECK_STATUS=$?

(
  cd "$REJECTED"
  "$PRESOLVE" plan --json
) > "$TMP/rejected-plan.json" 2> "$TMP/rejected-plan.err"
PLAN_STATUS=$?
set -e

if [[ "$CHECK_STATUS" -eq 0 ]]; then
  echo "expected rejected check --json to return non-zero" >&2
  exit 1
fi

if [[ "$PLAN_STATUS" -eq 0 ]]; then
  echo "expected rejected plan --json to return non-zero" >&2
  exit 1
fi

python3 - "$TMP/rejected-check.json" "$TMP/rejected-plan.json" <<'PY'
import json
import sys

with open(sys.argv[1], encoding="utf-8") as handle:
    check = json.load(handle)

with open(sys.argv[2], encoding="utf-8") as handle:
    plan = json.load(handle)

assert check["kind"] == "compatibility_check"
assert check["status"] == "incompatible"
assert check["problems"][0]["type"] == "missing_capability"
assert check["problems"][0]["code"] == "PS2001"
assert check["problems"][0]["interface"] == "presolve:kv/store"

assert plan["kind"] == "deployment_plan"
assert plan["status"] == "rejected"
assert plan["plan"] is None
assert plan["problems"][0]["type"] == "missing_capability"
assert plan["problems"][0]["code"] == "PS2001"
assert plan["problems"][0]["interface"] == "presolve:kv/store"
PY

echo "== documentation checks =="
(
  cd "$ROOT"
  python3 scripts/check-docs.py
)

echo "Chunk 20 inspection/planning checkpoint passed"

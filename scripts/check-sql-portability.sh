#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
APP="$ROOT/examples/sql-demo"
SQLITE_ENV="$ROOT/examples/portability-proof/sql-sqlite"
POSTGRES_ENV="$ROOT/examples/portability-proof/sql-postgres"
PRESOLVE="$ROOT/target/debug/presolve"
CHECKPOINT="$ROOT/target/debug/examples/sql_release_realization_checkpoint"
CONFORMANCE="$ROOT/target/debug/examples/sql_provider_conformance_checkpoint"
TMP="$(mktemp -d "${TMPDIR:-/tmp}/presolve-sql-portability.XXXXXX")"
RELEASE="$TMP/sql-demo.presolved"
RUN_ID="sql-$$"
POSTGRES_CONTAINER="presolve-sql-postgres-$$"
POSTGRES_IMAGE="postgres:18-alpine@sha256:77f585114c32fbca283dc835b0596f4e52b51b4c6662d7810b2f4084f60a1873"

export PRESOLVE_SQLITE_PATH="$TMP/sqlite.db"
export PRESOLVE_POSTGRES_DATABASE="presolve"
export PRESOLVE_POSTGRES_USER="presolve"
export PRESOLVE_POSTGRES_PASSWORD="presolve-proof-password"

cleanup() {
  docker rm -f "$POSTGRES_CONTAINER" >/dev/null 2>&1 || true
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

start_postgres() {
  docker run \
    --detach \
    --rm \
    --name "$POSTGRES_CONTAINER" \
    --env "POSTGRES_DB=$PRESOLVE_POSTGRES_DATABASE" \
    --env "POSTGRES_USER=$PRESOLVE_POSTGRES_USER" \
    --env "POSTGRES_PASSWORD=$PRESOLVE_POSTGRES_PASSWORD" \
    --publish 127.0.0.1::5432 \
    "$POSTGRES_IMAGE" >/dev/null

  local address
  address="$(docker port "$POSTGRES_CONTAINER" 5432/tcp | head -n 1)"
  if [[ -z "$address" ]]; then
    echo "could not determine PostgreSQL test endpoint" >&2
    exit 1
  fi

  export PRESOLVE_POSTGRES_HOST="${address%:*}"
  export PRESOLVE_POSTGRES_PORT="${address##*:}"

  for _ in $(seq 1 60); do
    if docker exec "$POSTGRES_CONTAINER" \
      pg_isready \
      --username "$PRESOLVE_POSTGRES_USER" \
      --dbname "$PRESOLVE_POSTGRES_DATABASE" >/dev/null 2>&1; then
      return
    fi
    sleep 1
  done

  echo "PostgreSQL test server did not become ready" >&2
  docker logs "$POSTGRES_CONTAINER" >&2 || true
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
  local output="$3"

  if ! "$CHECKPOINT" \
    "$mode" \
    "$RELEASE" \
    "$environment_file" \
    "$RUN_ID" > "$output" 2>&1; then
    cat "$output" >&2
    exit 1
  fi
}

require_command cargo
require_command docker
require_command python3

echo "Presolve relational SQL portability checkpoint"
echo "=============================================="
echo "Claim: one immutable SQL-capable release can resolve and execute through SQLite and PostgreSQL without SQL rewriting or an application rebuild."
echo

echo "[1/6] Build Presolve tooling"
cargo build --quiet --manifest-path "$ROOT/Cargo.toml" -p presolve-cli
cargo build --quiet --manifest-path "$ROOT/Cargo.toml" -p presolve-node --example sql_release_realization_checkpoint
cargo build --quiet --manifest-path "$ROOT/Cargo.toml" -p presolve-node --example sql_provider_conformance_checkpoint

echo "[2/6] Build one immutable SQL application release"
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
assert value["application"]["name"] == "sql-demo"
print(value["release"]["digest"])
PY
)"

printf "  release digest: %s\n" "$RELEASE_DIGEST"
printf "  archive sha256: %s\n" "$ARCHIVE_SHA_BEFORE"
echo

echo "[3/6] Resolve the same release against SQLite and PostgreSQL environments"
plan_release "$SQLITE_ENV" "$TMP/sqlite-plan.json"
plan_release "$POSTGRES_ENV" "$TMP/postgres-plan.json"

python3 - "$TMP/sqlite-plan.json" "$TMP/postgres-plan.json" "$RELEASE" "$RELEASE_DIGEST" <<'PY'
import json
from pathlib import Path
import sys

sqlite_path, postgres_path, release_path, expected_digest = sys.argv[1:]
release = str(Path(release_path).resolve())
expected = [
    (json.loads(Path(sqlite_path).read_text()), "sql-sqlite", "sqlite-sql"),
    (json.loads(Path(postgres_path).read_text()), "sql-postgres", "postgres-sql"),
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
    assert binding["interface"] == "presolve:sql/database"
    assert binding["provider_name"] == provider_name
    rows.append((environment_name, provider_name, binding["provider_id"]))

assert rows[0][2] != rows[1][2]
print(f"  {'environment':20} {'selected provider':18} plan")
print(f"  {'-' * 20} {'-' * 18} {'--------'}")
for environment, provider, _ in rows:
    print(f"  {environment:20} {provider:18} resolved")
PY

echo

echo "[4/6] Require both concrete engines to pass the same SQL conformance harness"
start_postgres
"$CONFORMANCE" sqlite
"$CONFORMANCE" postgres

echo
echo "[5/6] Execute those exact release bytes through both SQL engines"
execute_release \
  sqlite \
  "$SQLITE_ENV/presolve.env.toml" \
  "$TMP/sqlite-execution.txt"
execute_release \
  postgres \
  "$POSTGRES_ENV/presolve.env.toml" \
  "$TMP/postgres-execution.txt"

python3 - "$TMP/sqlite-execution.txt" "$TMP/postgres-execution.txt" "$RELEASE_DIGEST" "$RUN_ID" <<'PY'
from pathlib import Path
import sys

sqlite_path, postgres_path, expected_digest, run_id = sys.argv[1:]
expected = [
    (sqlite_path, "sql-sqlite", "sqlite-sql"),
    (postgres_path, "sql-postgres", "postgres-sql"),
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
    assert values["output"] == f"sql:{run_id}"
    print(f"  {environment:20} {provider:18} {values['output']}")
PY

echo

echo "[6/6] Verify application identity never changed"
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
  echo "release digest changed during SQL portability checkpoint" >&2
  exit 1
fi

if [[ "$ARCHIVE_SHA_AFTER" != "$ARCHIVE_SHA_BEFORE" ]]; then
  echo "encoded .presolved bytes changed during SQL portability checkpoint" >&2
  exit 1
fi

printf "  release digest unchanged: yes (%s)\n" "$RELEASE_DIGEST"
printf "  archive sha256 unchanged: yes (%s)\n" "$ARCHIVE_SHA_BEFORE"
echo
echo "PASS — one .presolved SQL release, two conforming engines, two plans, zero application rebuilds."

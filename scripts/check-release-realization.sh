#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
APP="$ROOT/examples/kv-demo"
LOCAL_ENV="$ROOT/examples/portability-proof/local-memory/presolve.env.toml"
CLOUDFLARE_ENV="$ROOT/examples/portability-proof/managed-cloudflare/presolve.env.toml"
PRESOLVE="$ROOT/target/debug/presolve"
TMP="$(mktemp -d "${TMPDIR:-/tmp}/presolve-release-realization.XXXXXX")"
RELEASE="$TMP/kv-demo.presolved"
COMPONENT="$TMP/kv-demo.wasm"
RUN_ID="proof-$$"
LIVE_CLOUDFLARE=false

if [[ "${1:-}" == "--cloudflare" ]]; then
  LIVE_CLOUDFLARE=true
elif [[ $# -ne 0 ]]; then
  echo "usage: $0 [--cloudflare]" >&2
  exit 2
fi

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

extract_component() {
  python3 - "$1" "$2" <<'PY'
import json
from pathlib import Path
import sys
import tarfile

release = Path(sys.argv[1])
output = Path(sys.argv[2])

with tarfile.open(release, "r:*") as archive:
    manifest_file = archive.extractfile("manifest.json")
    if manifest_file is None:
        raise SystemExit("release archive is missing manifest.json")

    manifest = json.load(manifest_file)
    workloads = manifest.get("workloads", [])
    if len(workloads) != 1:
        raise SystemExit(
            f"release realization checkpoint requires one workload, found {len(workloads)}"
        )

    workload = workloads[0]
    if workload.get("kind") != "component":
        raise SystemExit(
            f"release realization checkpoint requires a component workload, found {workload.get('kind')!r}"
        )

    artifacts = workload.get("artifacts", [])
    if len(artifacts) != 1:
        raise SystemExit(
            f"component workload requires one artifact, found {len(artifacts)}"
        )

    artifact_path = artifacts[0]["path"]
    component_file = archive.extractfile(artifact_path)
    if component_file is None:
        raise SystemExit(f"release archive is missing {artifact_path}")

    output.write_bytes(component_file.read())
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
extract_component "$RELEASE" "$COMPONENT"
test -f "$COMPONENT"

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
  "$COMPONENT" \
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
    "$COMPONENT" \
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
if [[ "$LIVE_CLOUDFLARE" == true ]]; then
  echo "same-release local + Cloudflare realization proof passed"
else
  echo "detached-release local realization checkpoint passed"
fi

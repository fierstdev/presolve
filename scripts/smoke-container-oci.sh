#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

for command in docker cargo python3; do
  if ! command -v "$command" >/dev/null 2>&1; then
    echo "missing required command: $command" >&2
    exit 1
  fi
done

docker info >/dev/null

SUFFIX="$$"
NETWORK="presolve-oci-checkpoint-${SUFFIX}"
REGISTRY_CONTAINER="presolve-oci-registry-${SUFFIX}"
BUILDER="presolve-oci-builder-${SUFFIX}"
METADATA="$(mktemp)"
PORT="$(
  python3 - <<'PY'
import socket

sock = socket.socket()
sock.bind(("127.0.0.1", 0))
print(sock.getsockname()[1])
sock.close()
PY
)"

INTERNAL_IMAGE="${REGISTRY_CONTAINER}:5000/presolve-capability-checkpoint:checkpoint"
HOST_REPOSITORY="127.0.0.1:${PORT}/presolve-capability-checkpoint"
DIGEST=""

cleanup() {
  set +e

  if [[ -n "$DIGEST" ]]; then
    docker image rm -f "${HOST_REPOSITORY}@${DIGEST}" >/dev/null 2>&1
  fi

  docker buildx rm -f "$BUILDER" >/dev/null 2>&1
  docker rm -f "$REGISTRY_CONTAINER" >/dev/null 2>&1
  docker network rm "$NETWORK" >/dev/null 2>&1
  rm -f "$METADATA"
}

trap cleanup EXIT INT TERM

echo "== create isolated local registry =="
docker network create "$NETWORK" >/dev/null
docker run \
  --detach \
  --name "$REGISTRY_CONTAINER" \
  --network "$NETWORK" \
  --publish "127.0.0.1:${PORT}:5000" \
  registry:2 >/dev/null

READY=0
for _ in $(seq 1 50); do
  if python3 - "$PORT" <<'PY' >/dev/null 2>&1
import sys
import urllib.request

port = int(sys.argv[1])
with urllib.request.urlopen(f"http://127.0.0.1:{port}/v2/", timeout=1) as response:
    if response.status != 200:
        raise SystemExit(1)
PY
  then
    READY=1
    break
  fi

  sleep 0.1
done

if [[ "$READY" != "1" ]]; then
  echo "local checkpoint registry did not become ready" >&2
  exit 1
fi

echo "== build OCI workload into registry =="
docker buildx create \
  --name "$BUILDER" \
  --driver docker-container \
  --driver-opt "network=${NETWORK}" >/dev/null

docker buildx inspect \
  --builder "$BUILDER" \
  --bootstrap >/dev/null

docker buildx build \
  --builder "$BUILDER" \
  --provenance=false \
  --sbom=false \
  --metadata-file "$METADATA" \
  --output "type=registry,name=${INTERNAL_IMAGE},registry.insecure=true,oci-mediatypes=true" \
  examples/container-capability-checkpoint

DIGEST="$(
  python3 - "$METADATA" <<'PY'
import json
import sys

with open(sys.argv[1], "r", encoding="utf-8") as handle:
    metadata = json.load(handle)

print(metadata["containerimage.descriptor"]["digest"])
PY
)"

MEDIA_TYPE="$(
  python3 - "$METADATA" <<'PY'
import json
import sys

with open(sys.argv[1], "r", encoding="utf-8") as handle:
    metadata = json.load(handle)

print(metadata["containerimage.descriptor"]["mediaType"])
PY
)"

SIZE="$(
  python3 - "$METADATA" <<'PY'
import json
import sys

with open(sys.argv[1], "r", encoding="utf-8") as handle:
    metadata = json.load(handle)

print(metadata["containerimage.descriptor"]["size"])
PY
)"

echo "OCI descriptor:"
echo "  media type: $MEDIA_TYPE"
echo "  digest:     $DIGEST"
echo "  size:       $SIZE"

case "$MEDIA_TYPE" in
  application/vnd.oci.image.manifest.v1+json|application/vnd.oci.image.index.v1+json)
    ;;
  *)
    echo "unexpected non-OCI descriptor media type: $MEDIA_TYPE" >&2
    exit 1
    ;;
esac

echo "== load exact OCI artifact into local Docker image store =="
docker pull "${HOST_REPOSITORY}@${DIGEST}" >/dev/null

echo "== run Presolve end-to-end checkpoint =="
cargo run \
  -p presolve-node \
  --example oci_checkpoint \
  -- \
  "$DIGEST" \
  "$MEDIA_TYPE" \
  "$SIZE"

echo "== Chunk 16 checkpoint passed =="

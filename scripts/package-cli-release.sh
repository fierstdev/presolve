#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
OUTPUT_DIRECTORY="${1:-$ROOT/dist/release}"

cd "$ROOT"
mkdir -p "$OUTPUT_DIRECTORY"
OUTPUT_DIRECTORY="$(cd "$OUTPUT_DIRECTORY" && pwd)"

echo "== verify locked dependency graph =="
cargo metadata --locked --format-version 1 > /dev/null

echo "== build Presolve CLI release binary =="
cargo build --locked --release -p presolve-cli

BINARY="$ROOT/target/release/presolve"
if [[ ! -x "$BINARY" ]]; then
  echo "release binary was not produced at $BINARY" >&2
  exit 1
fi

VERSION_OUTPUT="$($BINARY --version)"
read -r BINARY_NAME VERSION EXTRA <<< "$VERSION_OUTPUT"

if [[ "$BINARY_NAME" != "presolve" || -z "$VERSION" || -n "${EXTRA:-}" ]]; then
  echo "unexpected version output: $VERSION_OUTPUT" >&2
  exit 1
fi

TARGET="$(rustc -vV | sed -n 's/^host: //p')"
if [[ -z "$TARGET" ]]; then
  echo "could not determine the native Rust target" >&2
  exit 1
fi

PACKAGE_NAME="presolve-v${VERSION}-${TARGET}"
STAGING_DIRECTORY="$OUTPUT_DIRECTORY/$PACKAGE_NAME"
ARCHIVE="$OUTPUT_DIRECTORY/$PACKAGE_NAME.tar.gz"
CHECKSUM="$ARCHIVE.sha256"

rm -rf "$STAGING_DIRECTORY"
rm -f "$ARCHIVE" "$CHECKSUM"
mkdir -p "$STAGING_DIRECTORY"

cp "$BINARY" "$STAGING_DIRECTORY/presolve"
cp "$ROOT/LICENSE" "$STAGING_DIRECTORY/LICENSE"

"$STAGING_DIRECTORY/presolve" --version

tar -C "$OUTPUT_DIRECTORY" -czf "$ARCHIVE" "$PACKAGE_NAME"
rm -rf "$STAGING_DIRECTORY"

python3 - "$ARCHIVE" "$CHECKSUM" <<'PY'
from hashlib import sha256
from pathlib import Path
import sys

archive = Path(sys.argv[1])
checksum = Path(sys.argv[2])
digest = sha256(archive.read_bytes()).hexdigest()
checksum.write_text(f"{digest}  {archive.name}\n", encoding="utf-8")
PY

python3 - "$ARCHIVE" "$CHECKSUM" <<'PY'
from hashlib import sha256
from pathlib import Path
import sys

archive = Path(sys.argv[1])
checksum = Path(sys.argv[2])
expected, name = checksum.read_text(encoding="utf-8").strip().split(maxsplit=1)
if name != archive.name:
    raise SystemExit(f"checksum names {name!r}, expected {archive.name!r}")
actual = sha256(archive.read_bytes()).hexdigest()
if actual != expected:
    raise SystemExit("release archive checksum verification failed")
PY

echo "release package: $ARCHIVE"
echo "checksum:        $CHECKSUM"
echo "version:         $VERSION"
echo "target:          $TARGET"

if [[ -n "${GITHUB_OUTPUT:-}" ]]; then
  {
    echo "archive=$ARCHIVE"
    echo "checksum=$CHECKSUM"
    echo "version=$VERSION"
    echo "target=$TARGET"
    echo "package_name=$PACKAGE_NAME"
  } >> "$GITHUB_OUTPUT"
fi

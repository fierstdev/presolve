#!/usr/bin/env sh
set -eu

REPOSITORY="fierstdev/presolve"
VERSION="${PRESOLVE_VERSION:-0.1.0-alpha.1}"
INSTALL_DIR="${PRESOLVE_INSTALL_DIR:-$HOME/.presolve/bin}"
TAG="v${VERSION}"

fail() {
  printf 'presolve installer: %s\n' "$1" >&2
  exit 1
}

command -v curl >/dev/null 2>&1 || fail "curl is required"
command -v tar >/dev/null 2>&1 || fail "tar is required"

OS="$(uname -s)"
ARCH="$(uname -m)"

case "$OS" in
  Linux) OS_TARGET="unknown-linux-gnu" ;;
  Darwin) OS_TARGET="apple-darwin" ;;
  *) fail "unsupported operating system: $OS" ;;
esac

case "$ARCH" in
  x86_64|amd64) ARCH_TARGET="x86_64" ;;
  arm64|aarch64) ARCH_TARGET="aarch64" ;;
  *) fail "unsupported architecture: $ARCH" ;;
esac

TARGET="${ARCH_TARGET}-${OS_TARGET}"
PACKAGE="presolve-v${VERSION}-${TARGET}"
ARCHIVE="${PACKAGE}.tar.gz"
BASE_URL="https://github.com/${REPOSITORY}/releases/download/${TAG}"

TMP_DIR="$(mktemp -d 2>/dev/null || mktemp -d -t presolve-install)"
trap 'rm -rf "$TMP_DIR"' EXIT HUP INT TERM

printf 'Installing Presolve %s for %s...\n' "$VERSION" "$TARGET"
curl -fL --retry 3 --proto '=https' --tlsv1.2 "$BASE_URL/$ARCHIVE" -o "$TMP_DIR/$ARCHIVE"
curl -fL --retry 3 --proto '=https' --tlsv1.2 "$BASE_URL/$ARCHIVE.sha256" -o "$TMP_DIR/$ARCHIVE.sha256"

EXPECTED="$(awk '{print $1}' "$TMP_DIR/$ARCHIVE.sha256")"
[ -n "$EXPECTED" ] || fail "release checksum is empty"

if command -v sha256sum >/dev/null 2>&1; then
  ACTUAL="$(sha256sum "$TMP_DIR/$ARCHIVE" | awk '{print $1}')"
elif command -v shasum >/dev/null 2>&1; then
  ACTUAL="$(shasum -a 256 "$TMP_DIR/$ARCHIVE" | awk '{print $1}')"
else
  fail "sha256sum or shasum is required to verify the download"
fi

[ "$EXPECTED" = "$ACTUAL" ] || fail "release checksum verification failed"

tar -xzf "$TMP_DIR/$ARCHIVE" -C "$TMP_DIR"
[ -x "$TMP_DIR/$PACKAGE/presolve" ] || fail "release archive does not contain the Presolve binary"

mkdir -p "$INSTALL_DIR"
cp "$TMP_DIR/$PACKAGE/presolve" "$INSTALL_DIR/presolve"
chmod 755 "$INSTALL_DIR/presolve"

"$INSTALL_DIR/presolve" --version >/dev/null
printf 'Installed %s\n' "$INSTALL_DIR/presolve"

case ":${PATH:-}:" in
  *":$INSTALL_DIR:"*) ;;
  *)
    SHELL_NAME="$(basename "${SHELL:-sh}")"
    case "$SHELL_NAME" in
      zsh) RC_FILE="$HOME/.zshrc" ;;
      bash) RC_FILE="$HOME/.bashrc" ;;
      *) RC_FILE="" ;;
    esac

    PATH_LINE="export PATH=\"$INSTALL_DIR:\$PATH\""
    if [ -n "$RC_FILE" ]; then
      touch "$RC_FILE"
      if ! grep -F "$INSTALL_DIR" "$RC_FILE" >/dev/null 2>&1; then
        printf '\n# Presolve CLI\n%s\n' "$PATH_LINE" >> "$RC_FILE"
        printf 'Added Presolve to PATH in %s\n' "$RC_FILE"
      fi
    fi
    printf 'For this shell, run: export PATH="%s:$PATH"\n' "$INSTALL_DIR"
    ;;
esac

printf 'Run `presolve --version` to verify the installation.\n'

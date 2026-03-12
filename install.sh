#!/usr/bin/env sh
set -eu

BIN_NAME="aipoor"
REPO="${GITHUB_REPO:-lyair1/aipoor}"
VERSION="${VERSION:-}"

if [ -f "./Cargo.toml" ] && [ -d "./src" ]; then
  cargo install --path .
  "$HOME/.cargo/bin/$BIN_NAME" setup --project "$(pwd)"
  exit 0
fi

OS="$(uname -s | tr '[:upper:]' '[:lower:]')"
ARCH="$(uname -m)"

case "$ARCH" in
  x86_64) ARCH="x86_64" ;;
  arm64|aarch64) ARCH="aarch64" ;;
esac

if [ -z "$VERSION" ]; then
  VERSION="$(curl -fsSL "https://api.github.com/repos/$REPO/releases/latest" | sed -n 's/.*"tag_name": *"\([^"]*\)".*/\1/p' | head -n 1)"
fi

if [ -z "$VERSION" ]; then
  echo "Unable to determine release version. Set VERSION or install from a checkout." >&2
  exit 1
fi

ASSET="${BIN_NAME}-${OS}-${ARCH}.tar.gz"
URL="https://github.com/$REPO/releases/download/$VERSION/$ASSET"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

curl -fsSL "$URL" -o "$TMP_DIR/$ASSET"
tar -xzf "$TMP_DIR/$ASSET" -C "$TMP_DIR"

INSTALL_DIR="${INSTALL_DIR:-$HOME/.local/bin}"
mkdir -p "$INSTALL_DIR"
install "$TMP_DIR/$BIN_NAME" "$INSTALL_DIR/$BIN_NAME"

echo "Installed $BIN_NAME to $INSTALL_DIR/$BIN_NAME"
"$INSTALL_DIR/$BIN_NAME" setup --project "$(pwd)"

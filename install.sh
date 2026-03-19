#!/bin/bash
set -euo pipefail

REPO="olucasandrade/rsight"
INSTALL_DIR="${INSTALL_DIR:-$HOME/.local/bin}"

ARCH=$(uname -m)
case "$ARCH" in
  x86_64) TARGET="x86_64-apple-darwin" ;;
  arm64)  TARGET="aarch64-apple-darwin" ;;
  *) echo "Unsupported architecture: $ARCH" && exit 1 ;;
esac

VERSION=$(curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" \
  | grep '"tag_name"' | cut -d'"' -f4)

ARCHIVE="rsight-${VERSION#v}-${TARGET}.tar.gz"
URL="https://github.com/${REPO}/releases/download/${VERSION}/${ARCHIVE}"

echo "Installing rsight ${VERSION} for ${ARCH}..."
mkdir -p "$INSTALL_DIR"
curl -fsSL "$URL" | tar xz -C "$INSTALL_DIR"

echo "Installed to $INSTALL_DIR/rsight"
echo "Add to PATH if needed: export PATH=\"$INSTALL_DIR:\$PATH\""

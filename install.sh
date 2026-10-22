#!/usr/bin/env bash
set -e

REPO="antimyichael/loom"
INSTALL_DIR="${INSTALL_DIR:-$HOME/.local/bin}"

OS="$(uname -s | tr '[:upper:]' '[:lower:]')"
case "$OS" in
  darwin) OS="macos" ;;
  linux) OS="linux" ;;
  *)
    echo "Unsupported operating system: $OS"
    exit 1
    ;;
esac

ARCH="$(uname -m)"
case "$ARCH" in
  x86_64|amd64) ARCH="x86_64" ;;
  arm64|aarch64) ARCH="aarch64" ;;
  *)
    echo "Unsupported architecture: $ARCH"
    exit 1
    ;;
esac

TARGET="loom-${OS}-${ARCH}"
echo "Detected platform: ${TARGET}"

LATEST_TAG=$(curl -s "https://api.github.com/repos/${REPO}/releases/latest" | grep '"tag_name":' | sed -E 's/.*"([^"]+)".*/\1/')
if [ -z "$LATEST_TAG" ]; then
  LATEST_TAG="v0.2.1"
fi

URL="https://github.com/${REPO}/releases/download/${LATEST_TAG}/${TARGET}.tar.gz"

echo "Downloading Loom ${LATEST_TAG} for ${TARGET}..."
mkdir -p "$INSTALL_DIR"

TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

if curl -sSL "$URL" -o "$TMP_DIR/loom.tar.gz"; then
  tar -xzf "$TMP_DIR/loom.tar.gz" -C "$TMP_DIR"
  mv "$TMP_DIR/loom" "$INSTALL_DIR/loom"
  chmod +x "$INSTALL_DIR/loom"
  echo "✓ Loom successfully installed to ${INSTALL_DIR}/loom"
else
  echo "Download failed from $URL."
  echo "You can build directly with Rust by running: cargo install --path ."
  exit 1
fi

case ":$PATH:" in
  *":$INSTALL_DIR:"*) ;;
  *)
    echo ""
    echo "Add Loom to your PATH by adding this line to your ~/.zshrc or ~/.bashrc:"
    echo "  export PATH=\"\$PATH:${INSTALL_DIR}\""
    ;;
esac

echo ""
echo "Try running:"
echo "  loom --help"

#!/bin/bash

set -e

REPO="MemerGamer/nodash"
INSTALL_PATH="/usr/local/bin/nodash"

echo "📦 Installing nodash from GitHub releases..."

# Detect OS
OS="$(uname -s)"
case "$OS" in
    Linux*)  OS="linux" ;;
    Darwin*) OS="macos" ;;
    *) echo "❌ Unsupported OS: $OS" && exit 1 ;;
esac

# Detect ARCH
ARCH="$(uname -m)"
case "$ARCH" in
    x86_64) ARCH="x86_64" ;;
    arm64|aarch64) ARCH="arm64" ;;
    *) echo "❌ Unsupported architecture: $ARCH" && exit 1 ;;
esac

# Fetch latest release tag
echo "🔍 Fetching latest release version..."
TAG=$(curl -s "https://api.github.com/repos/$REPO/releases/latest" | grep '"tag_name":' | sed -E 's/.*"v([^"]+)".*/\1/')
if [ -z "$TAG" ]; then
    echo "❌ Failed to detect latest version."
    exit 1
fi

# Construct download URL
BINARY_URL="https://github.com/$REPO/releases/download/v$TAG/nodash-${OS}-${ARCH}"

# Download
echo "⬇️ Downloading binary for $OS/$ARCH (v$TAG)..."
curl -L "$BINARY_URL" -o /tmp/nodash
chmod +x /tmp/nodash

# Move to /usr/local/bin
echo "🚚 Installing to $INSTALL_PATH..."
sudo mv /tmp/nodash "$INSTALL_PATH"

# Verify install
if command -v nodash &> /dev/null; then
    echo "✅ nodash installed successfully!"
    echo "👉 Run 'nodash' to get started."
    echo "👉 Run 'nodash update' anytime to update."
else
    echo "⚠️ Installation complete but 'nodash' not found in PATH."
    echo "Make sure $INSTALL_PATH is in your PATH."
fi

#!/bin/bash

set -e

REPO="MemerGamer/nodash"
VERSION="v1.0.1"
FILENAME="nodash-linux-$VERSION"
URL="https://github.com/$REPO/releases/download/$VERSION/$FILENAME"

INSTALL_PATH="/usr/local/bin/nodash"

echo "📦 Downloading nodash $VERSION..."
curl -sSL "$URL" -o nodash

echo "🔐 Making it executable..."
chmod +x nodash

echo "🚚 Moving to $INSTALL_PATH (requires sudo)..."
sudo mv nodash "$INSTALL_PATH"

echo "✅ nodash $VERSION installed successfully!"
echo "👉 Run with: nodash"

echo "🔄 To update nodash, run:"
echo "nodash update"

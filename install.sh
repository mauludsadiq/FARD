#!/bin/sh
set -e

REPO="mauludsadiq/FARD"
BINARY="fardrun"
INSTALL_DIR="/usr/local/bin"

# Detect platform
OS=$(uname -s)
ARCH=$(uname -m)

if [ "$OS" = "Darwin" ] && [ "$ARCH" = "arm64" ]; then
    ASSET="fardrun-macos-arm64"
elif [ "$OS" = "Darwin" ] && [ "$ARCH" = "x86_64" ]; then
    ASSET="fardrun-macos-x86_64"
elif [ "$OS" = "Linux" ] && [ "$ARCH" = "x86_64" ]; then
    ASSET="fardrun-linux-x86_64"
else
    echo "Unsupported platform: $OS $ARCH"
    echo "Please build from source: cargo build --release --bin fardrun"
    exit 1
fi

# Get latest release tag
TAG=$(curl -sf "https://api.github.com/repos/$REPO/releases/latest" | grep '"tag_name"' | sed 's/.*"tag_name": *"\([^"]*\)".*/\1/')

if [ -z "$TAG" ]; then
    echo "Could not determine latest release. Check https://github.com/$REPO/releases"
    exit 1
fi

URL="https://github.com/$REPO/releases/download/$TAG/$ASSET"

echo "Installing fardrun $TAG for $OS/$ARCH..."
echo "Downloading from $URL"

TMP=$(mktemp)
curl -sfL "$URL" -o "$TMP"
chmod +x "$TMP"

if [ -w "$INSTALL_DIR" ]; then
    mv "$TMP" "$INSTALL_DIR/$BINARY"
else
    sudo mv "$TMP" "$INSTALL_DIR/$BINARY"
fi

echo ""
echo "fardrun installed to $INSTALL_DIR/$BINARY"
echo ""
fardrun --version

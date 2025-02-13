#!/usr/bin/env bash
set -e  # Exit on error

REPO="jasonnathan/skeletor"

# Detect OS
if [[ "$OSTYPE" == "darwin"* ]]; then
    OS="macos"
elif [[ "$OSTYPE" == "linux"* ]]; then
    OS="linux"
elif [[ "$OS" == "Windows_NT" || "$OSTYPE" == "msys" || "$OSTYPE" == "cygwin" ]]; then
    OS="windows"
else
    echo "❌ Unsupported OS: $OSTYPE"
    exit 1
fi

# Detect Architecture
if [[ "$OS" == "windows" ]]; then
    ARCH=$(wmic os get osarchitecture | grep -Eo '64-bit|32-bit' || echo "unknown")
    if [[ "$ARCH" == "64-bit" ]]; then ARCH="x86_64"; else ARCH="unknown"; fi
else
    ARCH=$(uname -m)
fi

# Map architecture for releases
case "$ARCH" in
    x86_64) ARCH="x86_64" ;;
    arm64|aarch64) ARCH="aarch64" ;;
    *) echo "❌ Unsupported architecture: $ARCH"; exit 1 ;;
esac

# Determine platform target
BINARY="skeletor"
TARGET=""
EXT="tar.gz"

if [[ "$OS" == "macos" ]]; then
    TARGET="x86_64-apple-darwin"
elif [[ "$OS" == "linux" ]]; then
    TARGET="x86_64-unknown-linux-gnu"
elif [[ "$OS" == "windows" ]]; then
    TARGET="x86_64-pc-windows-msvc"
    EXT="zip"
else
    echo "❌ Unsupported OS detected."
    exit 1
fi

# Fetch latest version
VERSION=$(curl -s "https://api.github.com/repos/${REPO}/releases/latest" | grep -oP '"tag_name": "\K(.*?)(?=")')
if [[ -z "$VERSION" ]]; then
    echo "❌ Failed to determine latest version. Check GitHub Releases."
    exit 1
fi

# Define asset URL
ASSET="${BINARY}-${OS}-${TARGET}.${EXT}"
URL="https://github.com/${REPO}/releases/download/${VERSION}/${ASSET}"

echo "🔽 Downloading ${BINARY} ${VERSION} for ${OS}/${ARCH}..."
curl -L --fail "$URL" -o "/tmp/${ASSET}"

# Extract & Install
if [[ "$OS" == "windows" ]]; then
    echo "📦 Extracting Windows binary..."
    INSTALL_DIR="/c/Program Files/Skeletor"
    mkdir -p "$INSTALL_DIR"
    unzip -o "/tmp/${ASSET}" -d "$INSTALL_DIR"
    echo "✅ Installed to $INSTALL_DIR"
    echo "🔧 Add '$INSTALL_DIR' to your PATH if necessary."
else
    echo "📦 Extracting binary..."
    tar -xzf "/tmp/${ASSET}" -C "/tmp"
    chmod +x "/tmp/${BINARY}"

    # Install to /usr/local/bin
    INSTALL_DIR="/usr/local/bin"
    if [[ -w "$INSTALL_DIR" ]]; then
        mv "/tmp/${BINARY}" "$INSTALL_DIR/${BINARY}"
    else
        sudo mv "/tmp/${BINARY}" "$INSTALL_DIR/${BINARY}"
    fi

    echo "✅ Installed to $INSTALL_DIR"
fi

# Cleanup
rm "/tmp/${ASSET}"

echo "🎉 Installation complete! Run 'skeletor --help' to get started."

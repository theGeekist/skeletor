#!/usr/bin/env bash
# filepath: install.sh
set -e

REPO="jasonnathan/skeletor"

# Dynamically fetch the latest version
VERSION=$(curl -s "https://api.github.com/repos/${REPO}/releases/latest" | grep -oP '"tag_name": "\K(.*?)(?=")')
if [[ -z "$VERSION" ]]; then
    echo "Failed to determine latest version. Check GitHub Releases."
    exit 1
fi

# Determine OS and architecture
OS=$(uname -s | tr '[:upper:]' '[:lower:]')
ARCH=$(uname -m)

case "$ARCH" in
    x86_64) ARCH="x86_64" ;;
    arm64|aarch64) ARCH="aarch64" ;;
    *) echo "Unsupported architecture: $ARCH" ; exit 1 ;;
esac

# Construct the binary asset name
BINARY="skeletor"
TARGET=""
if [[ "$OS" == "darwin" ]]; then
    TARGET="x86_64-apple-darwin"
elif [[ "$OS" == "linux" ]]; then
    TARGET="x86_64-unknown-linux-gnu"
else
    echo "Unsupported OS: $OS"
    exit 1
fi

ASSET="${BINARY}-${OS}-latest-${TARGET}.tar.gz"
URL="https://github.com/${REPO}/releases/download/${VERSION}/${ASSET}"

echo "Downloading ${BINARY} ${VERSION} for ${OS}/${ARCH}..."
curl -L --fail "$URL" -o /tmp/"${ASSET}"

echo "Extracting binary..."
tar -xzf /tmp/"${ASSET}" -C /tmp
chmod +x /tmp/"${BINARY}"

# Move binary to /usr/local/bin (ensure permissions)
INSTALL_DIR="/usr/local/bin"
if [ -w "$INSTALL_DIR" ]; then
    mv /tmp/"${BINARY}" "$INSTALL_DIR/${BINARY}"
else
    sudo mv /tmp/"${BINARY}" "$INSTALL_DIR/${BINARY}"
fi

rm /tmp/"${ASSET}"
echo "Installation complete! Run 'skeletor --help' to get started."

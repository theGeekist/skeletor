#!/usr/bin/env bash
set -e

REPO="jasonnathan/skeletor"
HOMEBREW_TAP="jasonnathan/homebrew-tap"
VERSION="$1"

if [ -z "$VERSION" ]; then
  echo "Usage: $0 <version>"
  exit 1
fi

# Fetch latest release assets
echo "🔍 Fetching release assets for v$VERSION..."
wget -q "https://github.com/$REPO/releases/download/v$VERSION/skeletor-macos-latest-x86_64-apple-darwin.tar.gz"

echo "🔢 Calculating SHA256 checksum..."
CHECKSUM=$(shasum -a 256 skeletor-macos-latest-x86_64-apple-darwin.tar.gz | awk '{print $1}')
rm skeletor-macos-latest-x86_64-apple-darwin.tar.gz

echo "🔄 Updating skeletor.rb..."
sed -i.bak "s|url \".*\"|url \"https://github.com/$REPO/releases/download/v$VERSION/skeletor-macos-latest-x86_64-apple-darwin.tar.gz\"|" skeletor.rb
sed -i.bak "s|sha256 \".*\"|sha256 \"$CHECKSUM\"|" skeletor.rb
rm skeletor.rb.bak

# Commit & push update
echo "📦 Committing updated Homebrew formula..."
git add skeletor.rb
git commit -m "Update Skeletor to v$VERSION"
git push origin main

echo "✅ Homebrew formula updated!"

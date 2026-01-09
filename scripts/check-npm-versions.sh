#!/usr/bin/env bash
set -euo pipefail

TAG="${1#v}"
echo "Checking invalid package versions against tag: $TAG"

FAILED=false

# Check main package
MAIN_PKG="npm/skeletor/package.json"
if [[ -f "$MAIN_PKG" ]]; then
  PKG_VERSION=$(grep '"version":' "$MAIN_PKG" | head -n 1 | awk -F'"' '{print $4}')
  if [[ "$PKG_VERSION" != "$TAG" ]]; then
    echo "❌ $MAIN_PKG version $PKG_VERSION does not match tag $TAG"
    FAILED=true
  else
    echo "✅ $MAIN_PKG version $PKG_VERSION matches tag"
  fi
fi

# Check platform packages
for pkg in npm/platforms/*/package.json; do
  if [[ -f "$pkg" ]]; then
    PKG_VERSION=$(grep '"version":' "$pkg" | head -n 1 | awk -F'"' '{print $4}')
    if [[ "$PKG_VERSION" != "$TAG" ]]; then
      echo "❌ $pkg version $PKG_VERSION does not match tag $TAG"
      FAILED=true
    else
      echo "✅ $pkg version $PKG_VERSION matches tag"
    fi
  fi
done

if [[ "$FAILED" == "true" ]]; then
  echo "❌ Version mismatch detected."
  exit 1
fi

echo "✅ All npm package versions match tag $TAG"

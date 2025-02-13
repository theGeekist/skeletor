#!/usr/bin/env bash
set -e  # Exit immediately on error

# Get new version number
if [ -z "$1" ]; then
  echo "Usage: $0 <new_version>"
  exit 1
fi
NEW_VERSION="$1"

# Ensure we are on main branch and up to date
git checkout main
git pull origin main

echo "🔄 Updating Cargo.toml..."
sed -i.bak "s/^version = \".*\"/version = \"${NEW_VERSION}\"/" Cargo.toml
rm Cargo.toml.bak

echo "🔄 Updating version in main.rs..."
sed -i.bak "s/clap::Command::new(\"skeletor\").version(\".*\")/clap::Command::new(\"skeletor\").version(\"${NEW_VERSION}\")/" src/main.rs
rm src/main.rs.bak

# Commit changes
echo "📦 Committing version bump..."
git add Cargo.toml src/main.rs
git commit -m "Release v${NEW_VERSION}"

# Create git tag
echo "🏷️ Creating tag v${NEW_VERSION}..."
git tag "v${NEW_VERSION}"

# Push changes and tag
echo "🚀 Pushing changes & tag..."
git push origin main
git push origin "v${NEW_VERSION}"

echo "✅ Version bump complete! GitHub Actions will now build and release the binaries."

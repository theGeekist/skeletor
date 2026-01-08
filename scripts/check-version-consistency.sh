#!/bin/bash
# Version Consistency Checker for Skeletor
# Compares latest remote git tag with top CHANGELOG.md version
# Exit codes: 0 = consistent, 1 = drift detected, 2 = script error

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    local color=$1
    local message=$2
    echo -e "${color}${message}${NC}"
}

print_status $BLUE "🔍 Checking version consistency between git tags and changelog..."

# Get latest remote git tag
print_status $BLUE "📡 Fetching latest remote git tags..."
if ! git fetch origin --tags --quiet 2>/dev/null; then
    print_status $RED "❌ ERROR: Could not fetch remote tags"
    print_status $RED "This check requires access to remote repository to verify actual released versions"
    print_status $YELLOW "💡 Check your network connection and repository access"
    exit 2
fi

LATEST_TAG=$(git tag -l 'v0.*.*' | sort -V | tail -n1)
if [ -z "$LATEST_TAG" ]; then
    print_status $RED "❌ ERROR: No v0.*.* version tags found"
    exit 2
fi

# Remove 'v' prefix to get version number
LATEST_VERSION=${LATEST_TAG#v}
print_status $BLUE "🏷️  Latest v0.x.x git tag: $LATEST_TAG (version: $LATEST_VERSION)"

# Extract top version from CHANGELOG.md (first version after Unreleased section)
if [ ! -f "CHANGELOG.md" ]; then
    print_status $RED "❌ ERROR: CHANGELOG.md not found"
    exit 2
fi

# Find the first version line after "## [Unreleased] - ReleaseDate"
CHANGELOG_VERSION=$(awk '
    /^## \[Unreleased\] - ReleaseDate/ { found_unreleased = 1; next }
    found_unreleased && /^## \[[0-9]+\.[0-9]+\.[0-9]+\]/ {
        gsub(/^## \[/, "");
        gsub(/\].*/, "");
        print;
        exit
    }
' CHANGELOG.md)

MISSING_UNRELEASED=0
if [ -z "$CHANGELOG_VERSION" ]; then
    # Fallback: take the first version header in the file (release in progress)
    CHANGELOG_VERSION=$(awk '
        /^## \[[0-9]+\.[0-9]+\.[0-9]+\]/ {
            gsub(/^## \[/, "");
            gsub(/\].*/, "");
            print;
            exit
        }
    ' CHANGELOG.md)
    MISSING_UNRELEASED=1
fi

if [ -z "$CHANGELOG_VERSION" ]; then
    print_status $RED "❌ ERROR: Could not find version in CHANGELOG.md after ## [Unreleased] section"
    exit 2
fi

print_status $BLUE "📄 CHANGELOG.md top version: $CHANGELOG_VERSION"

# Compare versions
echo ""
print_status $BLUE "📊 Version Consistency Report"
echo "=============================="

if [ "$LATEST_VERSION" = "$CHANGELOG_VERSION" ]; then
    print_status $GREEN "✅ VERSIONS MATCH!"
    print_status $GREEN "🎯 CHANGELOG.md shows $CHANGELOG_VERSION as latest released (matches git tag v$LATEST_VERSION)"
    print_status $GREEN "🚀 Safe to proceed with commit/release"
    exit 0
fi

# Allow release-in-progress: changelog already bumped to Cargo.toml version
CARGO_VERSION=$(awk -F\" '/^version =/ {print $2; exit}' Cargo.toml)
if [ "$MISSING_UNRELEASED" -eq 1 ] && [ "$CHANGELOG_VERSION" = "$CARGO_VERSION" ]; then
    NEWEST=$(printf '%s\n' "$LATEST_VERSION" "$CHANGELOG_VERSION" | sort -V | tail -n1)
    if [ "$NEWEST" = "$CHANGELOG_VERSION" ] && [ "$CHANGELOG_VERSION" != "$LATEST_VERSION" ]; then
        print_status $YELLOW "⚠️  Release in progress: changelog version $CHANGELOG_VERSION matches Cargo.toml"
        print_status $YELLOW "🧩 Latest tag is v$LATEST_VERSION; proceeding with release"
        exit 0
    fi
fi

print_status $RED "❌ VERSION MISMATCH DETECTED!"
echo ""
print_status $RED "Expected latest version $LATEST_VERSION after Unreleased. Found $CHANGELOG_VERSION"
echo ""
print_status $YELLOW "💡 Resolution steps:"
echo "   1. CHANGELOG.md should show the latest released version ($LATEST_VERSION) as topmost"
echo "   2. Ensure git tag v$LATEST_VERSION exists and matches changelog"
echo "   3. Run 'cargo release' to properly synchronize versions"
echo ""
print_status $RED "🛑 Blocking commit/release due to version inconsistency"
exit 1

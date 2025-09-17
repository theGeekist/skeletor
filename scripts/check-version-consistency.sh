#!/bin/bash
# Version Consistency Checker for Skeletor
# This script detects version drift between Cargo.toml and source code
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

print_status $BLUE "🔍 Checking version consistency across codebase..."

# Get version from Cargo.toml
CARGO_VERSION=$(grep '^version = ' Cargo.toml | sed 's/version = "\(.*\)"/\1/')
if [ -z "$CARGO_VERSION" ]; then
    print_status $RED "❌ ERROR: Could not extract version from Cargo.toml"
    exit 2
fi

print_status $BLUE "📦 Cargo.toml version: $CARGO_VERSION"

# Files to check for hardcoded versions (excluding documentation)
SOURCE_FILES=(
    "src/lib.rs"
    "src/main.rs" 
    "tests/integration_test.rs"
    "CHANGELOG.md"
)

# Files to skip (documentation examples are allowed to have versions)
SKIP_FILES=(
    "README.md"
    "DEVELOPMENT.md"
    ".github/"
)

echo "🔎 Scanning for hardcoded versions in source code..."
version_issues=false

for file in "${SOURCE_FILES[@]}"; do
    if [ -f "$file" ]; then
        # Different patterns for different file types
        if [[ "$file" == "CHANGELOG.md" ]]; then
            # For CHANGELOG.md, we primarily rely on automation marker checks
            # Version numbers are expected in changelog history
            # Only flag obvious manual current version interference  
            hardcoded_versions=""
        else
            # For source files, look for hardcoded version strings  
            hardcoded_versions=$(grep -E "\b$CARGO_VERSION\b" "$file" | grep -v "env!" || true)
        fi
        
        if [ -n "$hardcoded_versions" ]; then
            echo "❌ $file: Found hardcoded version $CARGO_VERSION:"
            echo "$hardcoded_versions" | sed 's/^/   /'
            version_issues=true
        else
            echo "✅ $file: No hardcoded versions found"
        fi
    fi
done

# Special handling for CHANGELOG.md automation markers
if [ -f "CHANGELOG.md" ]; then
    echo "🔧 Verifying CHANGELOG.md automation markers..."
    if ! grep -q "<!-- next-header -->" "CHANGELOG.md"; then
        echo "❌ CHANGELOG.md: Missing <!-- next-header --> marker for cargo-release automation"
        version_issues=true
    fi
    if ! grep -q "<!-- next-url -->" "CHANGELOG.md"; then
        echo "❌ CHANGELOG.md: Missing <!-- next-url --> marker for cargo-release automation"
        version_issues=true
    fi
    if grep -q "<!-- next-header -->" "CHANGELOG.md" && grep -q "<!-- next-url -->" "CHANGELOG.md"; then
        echo "✅ CHANGELOG.md: Automation markers present for cargo-release"
    fi
fi

DRIFT_DETECTED=$version_issues

echo ""
echo "📄 README.md: Skipped (documentation examples allowed)"

echo ""
echo "🔧 Verifying lib.rs uses automatic versioning..."

if grep -q 'env!("CARGO_PKG_VERSION")' src/lib.rs; then
    echo "✅ lib.rs correctly uses env!(\"CARGO_PKG_VERSION\")"
else
    echo "❌ lib.rs does not use env!(\"CARGO_PKG_VERSION\") for version"
    DRIFT_DETECTED=true
fi

# Check integration test uses automatic versioning
if grep -q 'env!("CARGO_PKG_VERSION")' tests/integration_test.rs; then
    echo "✅ integration_test.rs correctly uses env!(\"CARGO_PKG_VERSION\")"
else
    echo "❌ integration_test.rs does not use env!(\"CARGO_PKG_VERSION\") for version"
    DRIFT_DETECTED=true
fi

# Final report
echo ""
echo "📊 Version Consistency Report"
echo "=============================="

if [ "$DRIFT_DETECTED" = true ]; then
    echo "❌ VERSION DRIFT DETECTED!"
    echo ""
    echo "🚨 CRITICAL: Manual version changes found in source code"
    echo "💡 Resolution steps:"
    echo "   1. Remove all hardcoded version numbers from source files"
    echo "   2. Use env!(\"CARGO_PKG_VERSION\") macro for automatic version sync"
    echo "   3. Update only Cargo.toml version (single source of truth)"
    echo "   4. For CHANGELOG.md, ensure cargo-release automation markers are present"
    echo "   5. Re-run this check to verify fixes"
    echo ""
    echo "🛑 Blocking commit/release to prevent version inconsistency"
    exit 1
else
    echo "✅ ALL VERSION CHECKS PASSED"
    echo "🎯 Version consistency verified: $CARGO_VERSION"
    echo "🚀 Safe to proceed with commit/release"
    exit 0
fi
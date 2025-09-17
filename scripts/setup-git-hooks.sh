#!/bin/bash
# Setup script to install git hooks for version drift protection
# Run this after cloning the repository: ./scripts/setup-git-hooks.sh

set -euo pipefail

echo "🔧 Setting up Skeletor development environment..."

# Check if we're in a git repository
if [ ! -d ".git" ]; then
    echo "❌ Error: Not in a git repository root"
    echo "Please run this from the skeletor project root directory"
    exit 1
fi

# Install pre-commit hook
echo "📋 Installing pre-commit hook for version consistency..."
if [ -f ".git/hooks/pre-commit" ]; then
    echo "⚠️  Existing pre-commit hook found - backing up to pre-commit.backup"
    mv .git/hooks/pre-commit .git/hooks/pre-commit.backup
fi

# Create symbolic link to our version check script
ln -s ../../scripts/pre-commit.sh .git/hooks/pre-commit
chmod +x .git/hooks/pre-commit

echo "✅ Pre-commit hook installed successfully"

# Test the version consistency checker
echo "🧪 Testing version consistency checker..."
if ./scripts/check-version-consistency.sh; then
    echo "✅ Version consistency check passed"
else
    echo "❌ Version consistency check failed"
    echo "Please fix any version drift issues before continuing development"
    exit 1
fi

echo ""
echo "🎉 Development environment setup complete!"
echo ""
echo "📝 What's been configured:"
echo "  ✅ Pre-commit hook: Blocks commits with version drift"
echo "  ✅ CI integration: Checks version consistency on every push"
echo "  ✅ Release protection: Prevents releases with version drift"
echo ""
echo "🚀 You're ready to develop! Version drift protection is now active."
echo ""
echo "💡 To manually run version check: ./scripts/check-version-consistency.sh"
#!/bin/bash
# Pre-commit hook to check version consistency
# Install this hook with: ln -s ../../scripts/pre-commit.sh .git/hooks/pre-commit

echo "🔍 Running pre-commit version consistency check..."

# Run the version consistency checker
if ! ./scripts/check-version-consistency.sh; then
    echo ""
    echo "💥 COMMIT BLOCKED: Version consistency check failed"
    echo "🔧 Please fix version drift issues before committing"
    echo ""
    echo "Quick fix commands:"
    echo "  git checkout -- src/lib.rs                    # Revert hardcoded version"
    echo "  git checkout -- tests/integration_test.rs     # Revert test version"
    echo ""
    exit 1
fi

echo "✅ Version consistency check passed - proceeding with commit"
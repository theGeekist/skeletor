# Skeletor Development Guide

## Overview
This document provides comprehensive development guidelines for the Skeletor CLI tool, focusing on version management, testing standards, and development workflows.

## 🔄 Version Management

### Automated Version System
Skeletor uses **automated version management** to prevent version drift across the codebase:

- **Single Source of Truth**: `Cargo.toml` contains the definitive version
- **Automatic Propagation**: `env!("CARGO_PKG_VERSION")` syncs version at compile time
- **Drift Protection**: Automated checks prevent manual version changes

### Version Consistency Enforcement

The codebase includes comprehensive version drift detection:

```bash
# Manual version check
./scripts/check-version-consistency.sh

# Automated enforcement points:
# ✅ Pre-commit hooks (optional developer setup)
# ✅ CI pipeline (mandatory - blocks builds with drift)
# ✅ Release process (mandatory - validates before publishing)
```

**What's checked:**
- **Source files**: No hardcoded version strings (must use `env!("CARGO_PKG_VERSION")`)
- **CHANGELOG.md**: Automation markers present (`<!-- next-header -->` and `<!-- next-url -->`)
- **Release config**: `pre-release-replacements` patterns configured for automatic CHANGELOG updates

### ⚠️ CRITICAL: Version Editing Rules

**NEVER manually edit version numbers or interfere with automation!**

- ✅ **Correct**: Edit version in `Cargo.toml` only
- ✅ **Correct**: Let cargo-release handle CHANGELOG.md updates automatically  
- ❌ **Wrong**: Manually editing `src/lib.rs`, `tests/`, etc.
- ❌ **Wrong**: Manually adding version entries to CHANGELOG.md
- ❌ **Wrong**: Removing cargo-release automation markers from CHANGELOG.md
- 🔧 **Tool**: Use `cargo release` for automated version updates

**Version locations in codebase**:
- `Cargo.toml` - **EDIT HERE ONLY** (single source of truth)
- `src/lib.rs` - Uses `env!("CARGO_PKG_VERSION")` macro (auto-sync)
- `tests/integration_test.rs` - Uses `env!("CARGO_PKG_VERSION")` for dynamic testing
- `CHANGELOG.md` - **AUTOMATED by cargo-release** (DO NOT manually edit version entries)
- `release.toml` - Contains `pre-release-replacements` patterns for CHANGELOG automation

## 📝 CHANGELOG.md Management

### Automated CHANGELOG Updates
Skeletor uses **cargo-release automation** for CHANGELOG.md:

- **Automation markers**: `<!-- next-header -->` and `<!-- next-url -->` enable cargo-release processing
- **Version replacement**: `pre-release-replacements` in `release.toml` automatically update version placeholders
- **Link management**: Release URLs and comparison links are automatically maintained

### CHANGELOG Workflow
```bash
# 1. Add changes under [Unreleased] section manually
# 2. Run release process - cargo-release automatically:
#    - Converts [Unreleased] to [version] - date
#    - Creates new [Unreleased] section for future changes  
#    - Updates comparison links with proper version tags
#    - Maintains automation markers for next release

# ❌ NEVER manually add version entries like ## [0.3.1] - 2024-01-01
# ✅ ALWAYS add content under existing ## [Unreleased] section
```

### CHANGELOG Structure
The CHANGELOG.md follows Keep a Changelog format with cargo-release automation:

```markdown
# Changelog
<!-- next-header -->
## [Unreleased] - ReleaseDate

### Added
- New features go here

### Changed  
- Changes go here

### Fixed
- Bug fixes go here

## [0.3.0] - 2024-12-19
<!-- Historical releases managed by cargo-release -->

<!-- next-url -->
[Unreleased]: https://github.com/theGeekist/skeletor/compare/v0.3.0...HEAD  
[0.3.0]: https://github.com/theGeekist/skeletor/releases/tag/v0.3.0
```

## 🧪 Development Setup

### Initial Environment Setup
```bash
# Clone and set up development environment
git clone https://github.com/theGeekist/skeletor.git
cd skeletor

# Install git hooks and configure environment
./scripts/setup-git-hooks.sh

# Build and test
cargo build
cargo test
```

### Development Workflow Protection

The setup script configures multiple layers of protection:

1. **Pre-commit Hook**: Prevents commits with version drift
2. **CI Integration**: Blocks builds if version inconsistency detected
3. **Release Validation**: Ensures version consistency before publishing

## 📋 Testing Standards

### Test Architecture
- **Shared Utilities**: Common test helpers in `src/test_utils.rs`
- **CLI Testing**: Use `create_*_matches()` functions for proper subcommand testing
- **Integration Tests**: End-to-end CLI validation in `tests/`
- **Coverage Target**: Maintain >80% test coverage

### Test Execution
```bash
# Run all tests
cargo test

# Run with coverage analysis
cargo llvm-cov --html

# Run specific test modules
cargo test apply::tests
cargo test snapshot::tests
cargo test integration_test
```

### Test Patterns

**✅ Correct CLI Testing**:
```rust
// Use helper functions for proper subcommand parsing
if let Some(sub_m) = create_apply_matches(args) {
    let result = run_apply(&sub_m);
    assert!(result.is_ok());
}
```

**❌ Incorrect CLI Testing**:
```rust
// Don't manually construct CLI matches
let matches = build_cli().get_matches_from(args);
// This can fail if args don't include subcommand properly
```

## 🔍 Code Quality

### Static Analysis
```bash
# Linting
cargo clippy -- -D warnings

# Formatting
cargo fmt --check

# Documentation tests
cargo test --doc
```

### Pre-Release Checklist
- [ ] All tests pass (`cargo test`)
- [ ] Version consistency verified (`./scripts/check-version-consistency.sh`)
- [ ] Code linted without warnings (`cargo clippy`)
- [ ] Code properly formatted (`cargo fmt`)
- [ ] Coverage maintained >80% (`cargo llvm-cov`)

## 🚀 Release Process

### Automated Releases
```bash
# Use cargo-release for automated versioning
cargo release patch --execute   # 0.3.1 → 0.3.2
cargo release minor --execute   # 0.3.1 → 0.4.0
cargo release major --execute   # 0.3.1 → 1.0.0
```

### Release Validation
The release process automatically:
1. Runs version consistency checks
2. Executes full test suite
3. Validates code quality (clippy)
4. Updates version in `Cargo.toml`
5. Creates git tags and pushes changes

**Manual version changes will break this process!**

## 🛠️ Available Scripts

| Script | Purpose | Usage |
|--------|---------|-------|
| `scripts/setup-git-hooks.sh` | Initial development setup | `./scripts/setup-git-hooks.sh` |
| `scripts/check-version-consistency.sh` | Manual version validation | `./scripts/check-version-consistency.sh` |
| `scripts/pre-commit.sh` | Git pre-commit hook | Auto-executed on `git commit` |

## 💡 Best Practices

### Version Management
- Always use `cargo release` for version updates
- Never hardcode version strings in source code
- Use `env!("CARGO_PKG_VERSION")` for dynamic version access
- Test version consistency before every commit

### Code Organization
- Keep modules focused and cohesive
- Use shared utilities to reduce test duplication
- Maintain clear separation between library and CLI code
- Follow Rust naming conventions and idioms

### Documentation
- Update CHANGELOG.md for user-facing changes
- Include examples for new CLI options
- Maintain professional tone in error messages
- Test all code examples in documentation

## 🆘 Troubleshooting

### Version Drift Issues
If version consistency check fails:
1. **DO NOT** manually edit version numbers in source files
2. Check what changes were made to version-related files
3. Use `git diff` to see unauthorized version changes
4. Revert manual version edits and use `cargo release` instead

### Test Failures
If CLI tests fail with "unexpected argument" errors:
1. Ensure tests use `create_*_matches()` helper functions
2. Verify argument arrays don't include subcommand names
3. Check that test arguments match CLI definition

### Git Hook Issues
If pre-commit hook blocks commits unexpectedly:
1. Run manual version check: `./scripts/check-version-consistency.sh`
2. Fix any reported version drift issues
3. Retry commit after fixing consistency

---

**Remember**: The automated version management system is designed to prevent common release issues. Trust the automation and avoid manual version editing!
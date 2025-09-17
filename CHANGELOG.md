# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

<!-- next-header -->

## [Unreleased] - ReleaseDate

### Added
- **Version Management System**: Comprehensive automated version consistency enforcement
  - `scripts/check-version-consistency.sh`: Drift detection with colored output and detailed reporting
  - `scripts/setup-git-hooks.sh`: Developer environment setup with pre-commit hooks
  - `scripts/pre-commit.sh`: Git hook for version consistency validation
  - CI/CD integration: Version drift detection blocks builds and releases
  - CHANGELOG.md automation: cargo-release integration with `pre-release-replacements`
- **Test Architecture Revolution**: Complete overhaul of testing infrastructure for maintainability and reliability
  - `src/test_utils.rs`: Comprehensive shared testing framework with 150+ lines of utilities
    - `TestFileSystem`: Standardized temporary directory and file management
    - `create_*_matches()`: Unified CLI argument parsing for all subcommands (apply, snapshot, info)
    - `assert_*()` helpers: Consistent success/failure validation patterns
    - YAML config generators: `create_test_config()`, `create_invalid_config()`, etc.
  - **CLI Testing Standardization**: Eliminated 200+ lines of duplicated test code across modules
    - Fixed fundamental CLI argument parsing issues causing test failures
    - Replaced manual `Command::new()` constructions with standardized helpers
    - Unified test patterns in `apply.rs`, `snapshot.rs`, `info.rs`, and `main.rs` test modules
    - Converted `config.rs` and `tasks.rs` from manual `tempdir()` to shared `TestFileSystem`
  - **Integration Test Enhancement**: Complete rewrite with proper CLI pipeline validation
    - Dynamic version verification using `env!("CARGO_PKG_VERSION")`
    - End-to-end testing for apply, snapshot, and info workflows
    - Comprehensive error handling and edge case coverage
  - **Code Cleanup**: Removed unused `output.rs` module (477 lines) that wasn't being utilized
- **Development Documentation**: Comprehensive guides for contributors
  - `DEVELOPMENT.md`: Complete development workflow and version management guide
  - CHANGELOG.md automation workflow documentation
  - Git hook setup and usage instructions

### Changed
- **Version Management**: Single source of truth approach with automated synchronization
  - `src/lib.rs`: Now uses `env!("CARGO_PKG_VERSION")` instead of hardcoded version strings
  - All version references automatically sync from `Cargo.toml` at compile time
  - CHANGELOG.md automation markers added for cargo-release integration
- **Test Suite Architecture**: Standardized patterns and improved maintainability
  - CLI test utilities centralized in `src/test_utils.rs` 
  - Consistent use of helper functions across all test modules
  - Enhanced integration tests with proper CLI argument parsing
- **Release Process**: Enhanced automation and validation
  - `release.toml`: Added pre-release-replacements for CHANGELOG.md automation
  - Version consistency checks integrated into release workflow
  - Pre-commit hooks available for local development

### Fixed
- **Version Drift Prevention**: Eliminated inconsistencies across codebase
  - Removed hardcoded version "0.3.1" from `src/lib.rs`
  - Fixed potential version mismatches between components
  - Prevented manual version editing conflicts with cargo-release
- **Test Infrastructure Revolution**: Resolved pervasive CLI testing issues and established maintainable patterns
  - **Fixed Broken CLI Tests**: Resolved fundamental argument parsing failures across all test modules
    - `apply.rs` tests: Fixed subcommand recognition and argument validation
    - `snapshot.rs` tests: Corrected complex argument patterns for ignore flags and dry-run
    - `info.rs` tests: Eliminated hardcoded CLI construction causing test instability  
    - `main.rs` tests: Standardized argument parsing for integration testing
  - **Eliminated Test Code Duplication**: Replaced 400+ lines of redundant CLI setup code
    - Removed duplicate `Command::new("Skeletor")` constructions from every test module
    - Consolidated repetitive `ArgMatches` creation into shared utilities
    - Standardized temporary file/directory management across all tests
  - **Resolved CLI Argument Inconsistencies**: Fixed subcommand argument handling issues
    - Corrected argument array construction preventing proper subcommand recognition
    - Fixed `create_*_matches()` usage patterns to ensure proper CLI parsing
    - Eliminated "unexpected argument" errors in integration tests
- **Development Workflow**: Streamlined setup and consistency enforcement
  - Automated git hook installation for version drift protection
  - CI pipeline enhanced with comprehensive version validation
  - Release process safeguarded against version inconsistencies

## [0.3.1] - 2024-12-19

### Changed
- **BREAKING**: CLI argument structure redesigned for better UX:
  - `apply` command: `CONFIG_FILE` is now a positional argument instead of `-i` flag
    - Old: `skeletor apply -i my-template.yml`
    - New: `skeletor apply my-template.yml`
  - `info` command: `CONFIG_FILE` is now a positional argument instead of `-i` flag
    - Old: `skeletor info -i my-template.yml`
    - New: `skeletor info my-template.yml`
  - `snapshot` command: ignore flag changed from `-I` to `-i` for consistency
    - Old: `skeletor snapshot -I "*.log"`
    - New: `skeletor snapshot -i "*.log"`
- All help documentation and examples updated to reflect new CLI syntax

### Fixed
- Test suite fully updated to work with new argument structure
- CLI flag consistency across all subcommands

## [0.3.0] - 2024-12-19

### Added
- **Library API**: Comprehensive Rust library with `SkeletorConfig` and `apply_config()` for programmatic usage
- **Reporter System**: Flexible output with `DefaultReporter` and `SilentReporter` traits
- **CLI Hygiene**: Professional output with selective emojis and lowercase prefixes (`error:`, `info:`, `tip:`)
- **Enhanced Testing**: Improved test coverage to 82.51% with comprehensive edge case validation
- **Colored Output**: Professional terminal formatting with battle-tested CLI conventions

### Changed
- **BREAKING**: Replaced all emoji symbols with professional equivalents following CLI hygiene standards:
  - 📁 `Dir:` for directory operations (emoji + lowercase prefix)
  - 📄 `File:` for file operations (emoji + lowercase prefix)
  - ℹ️ `info:` for informational messages (emoji + lowercase prefix)
  - ✅ `success:` for successful operations (emoji + lowercase prefix)
  - ⚠️ `warning:` for warnings (emoji + lowercase prefix)
  - `error:` for errors (no emoji, color conveys importance, lowercase)
  - 🚀 `start:` for operation initiation
  - ⚡ `progress:` for progress updates
  - � `snapshot:` for snapshot operations
  - � `output:` for file output indicators
  - ⏱️ `duration:` for timing information
- Modernized error messages with professional tone and actionable guidance using lowercase `tip:` prefix
- Enhanced dual CLI/library architecture supporting both interactive and programmatic usage
- Improved test coverage with comprehensive integration test suite

### Fixed
- Compilation errors in apply.rs due to duplicate function definitions
- Missing imports and unused dependency warnings
- Critical test failures in snapshot module
- All clippy warnings for production-ready code quality

## [0.2.23] - 2024-09-15

### Added
- Modernized release process with cargo-release tooling
- Clippy integration in CI pipeline
- Enhanced development documentation

### Changed
- Simplified release hooks and build process
- Updated repository URLs and organizational structure

### Fixed
- All clippy warnings resolved
- Release configuration corrections
- Development toolchain updates

---

## Migration Guide for v0.3.0

**CLI Changes:**
- Error messages now use lowercase prefixes: `error:`, `info:`, `tip:`
- Professional emoji usage maintained for visual appeal
- Verbose dry-run provides detailed operation listings

**New Library API:**
```rust
use skeletor::{SkeletorConfig, apply_config};

let config = SkeletorConfig::from_yaml_file("template.yml")?;
let result = apply_config(&config, Path::new("./output"), false, false)?;
```

<!-- next-url -->
[Unreleased]: https://github.com/theGeekist/skeletor/compare/v0.3.1...HEAD
[0.3.1]: https://github.com/thegeekist/skeletor/releases/tag/v0.3.1
[0.3.0]: https://github.com/thegeekist/skeletor/releases/tag/v0.3.0
[0.2.23]: https://github.com/thegeekist/skeletor/releases/tag/v0.2.23
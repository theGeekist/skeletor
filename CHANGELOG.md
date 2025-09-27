# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

<!-- next-header -->

## [Unreleased]

### Fixed
- **CRITICAL Data Corruption Bug**: Apply command `-o` flag incorrectly mapped to overwrite instead of output
  - **Root Cause**: CLI argument `-o` was assigned to `overwrite` instead of `output`, risking data loss
  - **Safety Impact**: Users expecting `-o` for output directory could accidentally overwrite existing files
  - **Solution**: Added proper `-o/--output` flag for specifying output directory, kept `--overwrite` long-only for safety
  - **Examples Updated**: Help text now shows correct usage: `skeletor apply -o ../new-project template.yml`

### Enhanced
- **Apply Command Output Control**: Added `-o/--output` flag for specifying target directory
  - **New Functionality**: `skeletor apply config.yml -o /path/to/target` creates files in specified directory
  - **Default Behavior**: Without `-o`, files are created in current directory (backward compatible)
  - **Safety First**: `--overwrite` remains long-only flag to prevent accidental data loss
  - **Integration**: Full integration with existing dry-run, verbose, and overwrite features
- **Snapshot Ignore Pattern Handling**: Dramatically improved robustness and user experience
  - **Graceful Degradation**: Invalid patterns in `.gitignore` files now show warnings and continue processing
  - **Professional Messaging**: Uses established Reporter system for consistent colored output
  - **Smart Validation**: Direct command-line patterns fail fast, file patterns skip invalid entries with helpful tips
  - **Shell Expansion Guidance**: Clear documentation and examples warning about unquoted glob patterns
  - **Error Prevention**: Help text prominently shows correct quoted usage vs incorrect shell expansion

### Added
- **Comprehensive Test Coverage**: Added 12+ new tests covering critical functionality
  - **Apply Tests**: `-o/--output` flag behavior, default directory handling, overwrite flag separation
  - **Snapshot Tests**: Invalid pattern handling, mixed valid/invalid patterns, reporter integration
  - **Integration Tests**: End-to-end CLI testing for output directory functionality
  - **Edge Cases**: Comprehensive validation of error conditions and recovery scenarios

### Documentation
- **Shell Expansion Warning**: Help text now clearly explains quoting requirements
  - **Good Examples**: `skeletor snapshot -i "*.log" -i "target/*" .`
  - **Bad Examples**: `skeletor snapshot -i *.log -i target/* .` # Shell expands patterns
  - **Visual Indicators**: ✓/✗ symbols clearly show correct vs incorrect usage
- **CLI Help Enhancement**: Improved examples showing proper flag usage and safety considerations

## [0.3.7] - 2025-09-18

## [0.3.6] - 2025-09-18

### Enhanced
- **Test Coverage Excellence**: Achieved 83.29% coverage (exceeding 80% target)
  - **Comprehensive Unit Tests**: 44 new tests added across critical modules
  - **Apply Module**: Enhanced coverage for binary files, ignore patterns, verbose modes, config validation
  - **Output Module**: Complete testing of verbose/non-verbose behavior, format variations, result types
  - **Tasks Module**: Thorough testing of CreationResult, overwrite behavior, progress logging, edge cases
  - **Integration Testing**: Full CLI functionality and library API validation
- **File Operation Reporting**: Comprehensive tracking and user feedback for file operations
  - **Skip Reporting**: Files skipped (already exist) now clearly listed with helpful tips
  - **Overwrite Reporting**: Files updated by `--overwrite` flag now tracked and displayed
  - **Smart List Management**: Shows first 3 items by default, all items with `--verbose`
  - **User Guidance**: Helpful tips suggest `--overwrite` for skipped files and `--verbose` for complete lists
  - **Professional UX**: Clear, actionable feedback without overwhelming default output

### Fixed
- **CRITICAL Security Issue**: Dangerous test behavior causing source file corruption
  - **Root Cause**: Tests using `--overwrite` in project root overwrote `src/main.rs` and `src/lib.rs`
  - **Safety Protocol**: All file-creating tests now use isolated temporary directories
  - **Git Recovery**: Source files restored from git history after accidental overwrite
  - **Prevention**: Enhanced safety protocols prevent future catastrophic overwrites
- **Test Execution Safety**: Fixed parallel test execution race conditions
  - **Issue**: Multiple tests changing current working directory simultaneously caused interference
  - **Solution**: CI and test execution now use `--test-threads=1` to prevent directory races
  - **Impact**: Tests now run safely without creating files in project root directory
  - **Safe Paths**: All test configurations use descriptive, non-conflicting paths like `test_output/` instead of `src/`
- **Test Infrastructure**: Robust testing framework with shared utilities
  - **TestFileSystem**: Isolated testing environments prevent project file corruption
  - **Helper Functions**: Standardized CLI testing patterns eliminate code duplication
  - **Error Handling**: Comprehensive edge case coverage and failure scenario testing
- **Version Consistency**: `check-version-consistency.sh` updated to correctly check against remote git tag
- **Output System**: Enhanced `SimpleApplyResult` and `CreationResult` for comprehensive file operation tracking
- **Type System**: Coordinated type system updates across `output.rs`, `tasks.rs`, `apply.rs`, and `lib.rs`

## [0.3.5] - 2025-09-18

### Fixed
- **Release Pipeline**: Resolved version conflict preventing package publication
  - Package version 0.3.4 already existed, requiring patch increment to 0.3.5
  - Fixed CI failures due to duplicate version publishing attempts

## [0.3.4] - 2025-09-18

### Added
- **Critical Safety Protocols**: Comprehensive protection against dangerous operations
  - **Mandatory Testing Directory**: `_ops/` directory for ALL skeletor testing operations
  - **Workspace Protection Rules**: NEVER operate in project root, always use isolated testing environments
  - **Safety Documentation**: Comprehensive safety guide in `_ops/README.md` with emergency protocols
  - **Forbidden Operations List**: Explicit guidelines to prevent catastrophic data loss
  - **GitIgnore Protection**: Safe testing directory configuration to prevent accidental commits

### Fixed
- **CI Integration Test Failures**: Resolved "No such file or directory" errors in CI environments
  - **Robust Binary Discovery**: Multi-location binary detection for diverse CI environments
  - **Cross-Platform Support**: Windows (.exe) and Unix binary detection
  - **Fallback Build System**: Automatic cargo build when binary not found
  - **Enhanced Error Reporting**: Clear troubleshooting messages for CI failures
- **Dead Code Warnings**: Resolved compiler warnings for clean releases
  - **SilentReporter Warning**: Added `#[allow(dead_code)]` for public API structure
  - **Clippy Compliance**: Fixed needless borrows and formatting issues
- **Release Pipeline Stability**: All 158 tests passing across Linux/Windows/macOS environments

## [0.3.2] - 2024-12-19

### Added
- **Enhanced Dry-Run Reporting**: Comprehensive preview functionality with professional formatting
  - **Comprehensive Operation Preview**: Shows operations → binary files → ignore patterns in consistent order
  - **Binary File Detection**: Automatically detects and reports binary files separately during dry-run
  - **Enhanced Ignore Pattern Reporting**: Clear display of ignored files with pattern matching details
  - **Unified Reporting Interface**: Consistent formatting across apply and snapshot commands
- **Enhanced Verbose Mode**: Detailed operation preview for apply command
  - **Operation Preview**: Shows complete list of operations before execution when using --verbose
  - **Professional Completion Summary**: Enhanced formatting with separator lines and checkmark emoji
  - **Duration Reporting**: Comprehensive timing information for operation awareness
- **Improved Test Coverage**: Significantly enhanced test coverage across core modules
  - **Config Module**: Comprehensive error handling, file operations, and edge case testing (98.96% coverage)
  - **Output Module**: Complete reporter functionality and formatting validation (90.59% coverage)
  - **Overall Coverage**: Improved from 70.76% to 90.53%, well above 80% requirement
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
- **Reporter System Enhancement**: Complete overhaul of output formatting and user experience
  - **Enhanced Reporter Trait**: Added `dry_run_preview_comprehensive` and `verbose_operation_preview` methods
  - **Consistent Dry-Run Formatting**: Unified approach across apply and snapshot commands
  - **Professional CLI Output**: Improved formatting with proper separators, emojis, and status indicators
  - **Operation Categorization**: Clear separation of operations, binary files, and ignore patterns
- **Apply Command Enhancement**: Improved user experience with verbose mode
  - **Pre-execution Preview**: Shows complete operation list when --verbose flag is used
  - **Enhanced Completion Reporting**: Professional summary with separator lines and completion status
  - **Improved Error Messaging**: Better context and formatting for operation failures
- **Code Quality Improvements**: Enhanced maintainability and adherence to standards
  - **Clippy Compliance**: Fixed needless borrows and removed unused functions
  - **Code Cleanup**: Removed unused `task_path` function and `write_colored_inline` helper
  - **Import Optimization**: Streamlined imports and removed unused dependencies
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
[0.3.7]: https://github.com/theGeekist/skeletor/compare/v0.3.6...HEAD
[0.3.6]: https://github.com/theGeekist/skeletor/compare/v0.3.5...v0.3.6
[0.3.5]: https://github.com/theGeekist/skeletor/compare/v0.3.4...v0.3.5
[0.3.4]: https://github.com/theGeekist/skeletor/compare/v0.3.2...v0.3.4
[0.3.2]: https://github.com/theGeekist/skeletor/compare/v0.3.1...v0.3.2
[0.3.1]: https://github.com/thegeekist/skeletor/releases/tag/v0.3.1
[0.3.0]: https://github.com/thegeekist/skeletor/releases/tag/v0.3.0
[0.2.23]: https://github.com/thegeekist/skeletor/releases/tag/v0.2.23
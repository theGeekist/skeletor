# Changelog

All### Changed
- **BREAKING**: CLI output format updated for professional environments:
  - Directory operations: `📁 Dir: path/` 
  - File operations: `📄 File: path (size)`
  - Messages: `error:`, `info:`, `tip:` (lowercase prefixes)
- **Verbose Mode**: Enhanced dry-run output with detailed operation listings
- **Error Handling**: Contextual error messages with actionable guidance to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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

[Unreleased]: https://github.com/thegeekist/skeletor/compare/v0.2.23...HEAD
[0.2.23]: https://github.com/thegeekist/skeletor/releases/tag/v0.2.23
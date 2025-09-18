# Skeletor Codebase Instructions

## ⚠️ CRITICAL SAFETY PROTOCOLS

### 🚨 NEVER OPERATE IN PROJECT ROOT
- **NEVER** run `skeletor apply` commands in the project root directory (`/Users/jasonnathan/Repos/@pipewrk/skeletor`)
- **NEVER** create YAML test files in the project root - they could overwrite source code with `--overwrite`
- **NEVER** leave the current working directory when testing skeletor functionality
- **ALWAYS** use the dedicated `_ops/` directory for ALL testing operations

### 🛡️ MANDATORY TESTING PROTOCOLS
- **ALL skeletor testing MUST occur in `_ops/` directory only**
- **Create `_ops/` directory**: `mkdir -p _ops && cd _ops` before any testing
- **All YAML configs**: Create in `_ops/` subdirectories, never in project root
- **Apply operations**: Use `--dry-run` first, then only apply in isolated `_ops/` environments
- **Test `--overwrite`**: Only in `_ops/` with disposable test files, NEVER in project root

### 🔒 WORKSPACE PROTECTION RULES
- **Current Directory**: Always `cd _ops` before any skeletor apply/snapshot operations
- **YAML Files**: Only create test configs in `_ops/test-configs/` or similar subdirectories
- **Source Code**: NEVER risk overwriting `src/`, `tests/`, or any project files
- **Emergency Check**: Before ANY apply operation, verify `pwd` shows `_ops` subdirectory

### 🚫 FORBIDDEN OPERATIONS
- ❌ `skeletor apply config.yml` in project root
- ❌ Creating YAML files that include `src/main.rs` or other source paths
- ❌ Using `--overwrite` anywhere except controlled `_ops/` test environments
- ❌ Running apply commands without first checking current directory safety

## Development Cadences

### Release Strategy
- **Automated Releases**: Use `cargo release` for all version updates
- **Version Drift Protection**: Always run `./scripts/check-version-consistency.sh` before commits
- **CHANGELOG Automation**: cargo-release manages CHANGELOG.md via `pre-release-replacements`
- **Release Readiness**: CI validates version consistency, tests, and clippy warnings

### Test Strategy  
- **Test Coverage**: Maintain >80% coverage with `cargo llvm-cov`
- **Shared Utilities**: Use `src/test_utils.rs` for consistent CLI testing patterns
- **Integration Focus**: End-to-end CLI validation in `tests/integration_test.rs`
- **Module Testing**: Unit tests co-located with implementation using standardized helpers

### Documentation Maintenance
- **Always Update**: `README.md`, `DEVELOPMENT.md`, `CHANGELOG.md` with each significant change
- **Remove Warnings**: Zero clippy warnings, clear error messages, up-to-date examples
- **Version Sync**: All examples and documentation automatically track `Cargo.toml` version

### Cognitive Load Management
- **Low Complexity**: Shared test utilities eliminate duplication
- **Clear Patterns**: Standardized CLI argument parsing via `create_*_matches()` 
- **Automated Workflows**: Version management, testing, and releases require minimal manual intervention
- **Self-Documenting**: Code patterns and utilities reduce onboarding complexity

### Community Standards
- **Professional CLI**: Lowercase prefixes (`error:`, `info:`, `tip:`), consistent emoji usage
- **Contributor Friendly**: `DEVELOPMENT.md` provides complete setup and workflow guidance
- **Quality Assurance**: Pre-commit hooks, CI integration, comprehensive test coverage
- **Release Transparency**: Detailed CHANGELOG.md with automation markers for cargo-release

## Architecture Overview

**Skeletor** is a Rust CLI tool for project scaffolding with three core subcommands: `apply`, `snapshot`, and `info`. The codebase follows a modular design with shared utilities and automated version management.

### Core Components

- **`main.rs`**: CLI entry point using `clap` subcommands, delegates to module handlers
- **`lib.rs`**: CLI builder function with `env!("CARGO_PKG_VERSION")` for automatic version sync
- **`config.rs`**: YAML configuration handling with `.skeletorrc` default and `directories` validation
- **`tasks.rs`**: File/directory creation logic using `Task` enum and breadth-first traversal
- **`apply.rs`**: Scaffolding execution with `--dry-run` and `--overwrite` support
- **`snapshot.rs`**: Reverse operation capturing existing structures to YAML with ignore patterns
- **`info.rs`**: Metadata extraction and display from configuration files
- **`errors.rs`**: Centralized error handling with `thiserror` for structured error types
- **`output.rs`**: Reporter system with `DefaultReporter` and `SilentReporter` for flexible output
- **`test_utils.rs`**: Shared testing framework with CLI utilities and standardized patterns

### Development Infrastructure

- **`scripts/check-version-consistency.sh`**: Version drift detection with colored output
- **`scripts/setup-git-hooks.sh`**: Development environment setup with pre-commit hooks  
- **`scripts/pre-commit.sh`**: Git hook for automated version consistency validation
- **`DEVELOPMENT.md`**: Comprehensive developer guide with workflows and best practices
- **`.github/workflows/CI.yml`**: Automated testing, coverage, and version validation

### YAML Configuration Pattern

The tool expects a specific YAML structure with `directories` as the root key:
```yaml
directories:
  src:
    main.rs: |
      fn main() {
          println!("Hello, world!");
      }
    lib.rs: ""
  tests:
    integration.rs: "// test content"
```

## Critical Development Workflows

### Build & Test
- **Build**: `cargo build --release`
- **Test**: `cargo test` with shared utilities in `src/test_utils.rs`
- **Coverage**: `cargo llvm-cov --html` for >80% coverage target
- **Linting**: `cargo clippy -- -D warnings` (zero warnings enforced)

### Version Management
- **⚠️ CRITICAL**: NEVER manually update version numbers! Use `cargo-release` only
- **Single source**: Version in `Cargo.toml`, auto-propagated via `env!("CARGO_PKG_VERSION")`
- **Drift detection**: `./scripts/check-version-consistency.sh` validates consistency
- **Release command**: `cargo release patch|minor|major --execute`

### Testing Framework
- **Shared utilities**: `src/test_utils.rs` with `TestFileSystem` and `create_*_matches()`
- **CLI testing**: Standardized patterns eliminate duplication across modules
- **Integration tests**: `tests/integration_test.rs` with dynamic version verification
- **Coverage target**: Maintain >80% with comprehensive edge case validation

## Project-Specific Patterns

### Task Processing
- Breadth-first traversal via `traverse_structure` in `tasks.rs`
- Progress logging every 1000 operations to avoid I/O overhead
- Automatic parent directory creation for files

### CLI Argument Handling
- All subcommands support `--dry-run` for safe preview
- Input defaults to `.skeletorrc` via `default_file_path()`
- Overwrite protection by default (explicit `--overwrite` required)
- Positional arguments for config files (v0.3.1+)

### Error Handling & Output
- `SkeletorError` enum with `thiserror` for structured errors
- Professional CLI output: lowercase prefixes (`error:`, `info:`, `tip:`)
- Graceful degradation: warn on individual failures, continue processing
- Reporter system: `DefaultReporter` for colored output, `SilentReporter` for programmatic use

## Dependencies & Integration

### Key External Crates
- **`clap`**: CLI parsing with derive features and subcommand support
- **`serde_yaml`**: YAML processing expecting `Value::Mapping` for directories
- **`globset`**: Pattern matching for snapshot ignore functionality
- **`chrono`**: Timestamp generation for snapshot metadata
- **`thiserror`**: Structured error handling with context preservation
- **`tempfile`**: Isolated test environments and temporary file management
- **`termcolor`**: Professional colored output with CLI hygiene standards

### File Structure Conventions
- Source modules flat in `src/` (no subdirectories)
- Each feature in own module: `apply`, `snapshot`, `info`, `output`
- Shared utilities: `config.rs`, `tasks.rs`, `test_utils.rs`
- Tests co-located with implementation using standardized helpers
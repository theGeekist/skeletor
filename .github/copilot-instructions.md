# Skeletor Codebase Instructions

## Architecture Overview

**Skeletor** is a Rust CLI tool for project scaffolding with three core subcommands: `apply`, `snapshot`, and `info`. The codebase follows a modular design with each major feature in its own module.

### Core Components

- **`main.rs`**: CLI argument parsing using `clap` with subcommands. Version is hardcoded and should match `Cargo.toml`
- **`config.rs`**: YAML configuration handling. Default config file is `.skeletorrc` with required `directories` key
- **`tasks.rs`**: Core logic for file/directory creation. Uses `Task` enum (Dir/File) and breadth-first traversal
- **`apply.rs`**: Executes scaffolding from YAML config. Supports `--dry-run` and `--overwrite` flags
- **`snapshot.rs`**: Reverse operation - captures existing folder structure into YAML config
- **`errors.rs`**: Centralized error handling using `thiserror` crate

### YAML Configuration Pattern

The tool expects a specific YAML structure:
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
- **Test**: `cargo test` (comprehensive unit tests in each module)
- **Coverage**: Uses `cargo-llvm-cov` in CI (see `.github/workflows/CI.yml`)

### Performance Testing
- **`100k.yml`**: Large test file with 5000+ lines for performance validation
- Progress logging every 1000 tasks to avoid I/O overhead (see `tasks.rs:76`)

### Release Process
- **Single source of truth**: Version defined in `Cargo.toml`, manually synced to `main.rs:17`
- **Versioning scheme**: Follows 0.x.y pre-1.0 semver convention for Rust CLIs
- **`release.sh`**: Automated release script
- **`install.sh`**: Installation script for end users

## Project-Specific Patterns

### Task Processing
- Uses breadth-first traversal (`traverse_structure` in `tasks.rs`)
- Batch processing with progress logging every 1000 operations
- Parent directory creation is automatic for files

### Glob Pattern Handling
- Snapshot mode supports `.gitignore`-style patterns via `globset` crate
- Patterns can be file paths (reads content) or direct glob strings
- Binary file detection for content inclusion decisions

### Error Handling
- Custom `SkeletorError` enum wraps common error types (IO, YAML, Config)
- Graceful degradation: warns on individual file failures, continues processing

### CLI Design Conventions
- All subcommands support `--dry-run` for safe preview
- Input defaults to `.skeletorrc` if not specified via `default_file_path()`
- Overwrite protection by default (explicit `--overwrite` flag required)

## Dependencies & Integration

### Key External Crates
- **`clap`**: CLI parsing with derive features
- **`serde_yaml`**: YAML processing (expects `Value::Mapping` for directories)
- **`globset`**: Pattern matching for ignore functionality
- **`chrono`**: Timestamp generation for snapshots
- **`thiserror`**: Structured error handling

### Testing Framework
- Uses `tempfile` for isolated test environments
- Each module has comprehensive unit tests
- CLI argument parsing has dedicated test coverage

## File Structure Conventions

- Source modules are flat in `src/` (no subdirectories)
- Each feature gets its own module (`apply`, `snapshot`, `info`)
- Shared utilities in `config.rs` and `tasks.rs`
- Test files co-located with implementation
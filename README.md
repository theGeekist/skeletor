# Skeletor
![Build Status](https://github.com/theGeekist/skeletor/actions/workflows/CI.yml/badge.svg)
[![codecov](https://codecov.io/gh/theGeekist/skeletor/branch/main/graph/badge.svg)](https://codecov.io/gh/theGeekist/skeletor)

<p align="center">
  <strong>A blazing-fast Rust scaffolding tool.</strong><br>
    Generate thousands of files and directories with file contents from a <code>YAML</code> configuration in milliseconds.
<br>
  <small>Capture existing nested folders as YAML templates with snapshot mode.</small>
</p>


## Usage
Skeletor simplifies **project scaffolding** with an easy-to-use YAML configuration.


### Generate Files and Directories

#### With a `.skeletorrc` file
```bash
skeletor apply
```
#### Using a custom YAML config
```bash
skeletor apply -i custom.yml
```

### Example .skeletorrc Configuration
Create a YAML file (`.skeletorrc`) to define the directory structure:

```yaml
directories:
  src:
    main.rs: |
      fn main() {
          println!("Hello, Skeletor!");
      }
    lib.rs: ""
  tests:
    integration.rs: |
      #[test]
      fn sample_test() {
          assert_eq!(2 + 2, 4);
      }
  Cargo.toml: |
    [package]
    name = "my_project"
    version = "0.1.0"
```

**Apply the Configuration**
```bash
skeletor apply
```
This will **generate the entire folder structure** instantly!

**Preview Before Running**
```bash
# Quick summary of what would be created
skeletor apply --dry-run

# Detailed listing of all operations (useful for debugging)
skeletor apply --dry-run --verbose
```

## Installation

### Option 1: Install via Script (Linux/macOS)
```bash
curl -fsSL https://raw.githubusercontent.com/jasonnathan/skeletor/main/install.sh | bash
```
**tip:** Review the script before running to ensure security.  

### Option 2: Homebrew (macOS & Linux)
```bash
brew tap jasonnathan/skeletor
brew install skeletor
```
Easiest method if Homebrew is installed. 

### Option 3: Cargo (Recommended for Rust Users)
```bash
cargo install skeletor
```
Installs directly from crates.io.

### Option 4: Build from Source 
```bash
git clone https://github.com/jasonnathan/skeletor.git
cd skeletor
cargo install --path .
```
Rust & Cargo need to be installed on your system.  

## Key Features
- Generate Nested Files & Directories Instantly
- Dry-Run Mode – Preview before applying
- Snapshot Mode – Convert an existing folder into YAML
- Ignore Patterns & Binary File Detection
- Metadata & Stats Included

## Snapshot Mode
Capture a YAML snapshot of an existing folder.

**Create a Snapshot**
```bash
# Print YAML to stdout
skeletor snapshot .

# Save to file
skeletor snapshot . -o my-template.yml
```

**Ignore files and add a note**
```bash
skeletor snapshot -n "Initial snapshot" -I .gitignore -I .git/ .
```

**Preview Before Creating**
```bash
# Quick summary of what would be captured
skeletor snapshot --dry-run .

# Detailed listing with ignore pattern matching
skeletor snapshot --dry-run --verbose .
```

**Common Options**
- `-o custom.yml` → Save snapshot to file (prints to stdout if omitted)
- `-I "*.log"` → Exclude files based on patterns (can be used multiple times)
- `-I .gitignore` → Use .gitignore file patterns for exclusion
- `-n "Initial snapshot"` → Add custom notes to the snapshot
- `--include-contents` → Include file contents for text files (binary files will be empty)
- `--verbose` → Show detailed ignore pattern matching and file processing info

## Info Mode
Display metadata from a `.skeletorrc` file.

```bash
# Show info for .skeletorrc
skeletor info

# Show info for custom file
skeletor info -i my-template.yml
```

## Library Usage
Skeletor can be used as a Rust library for programmatic scaffolding in your applications.

### Add to Cargo.toml
```toml
[dependencies]
skeletor = "0.2"
```

### Basic Usage
```rust
use skeletor::{SkeletorConfig, apply_config};
use std::path::Path;

// Load configuration from YAML string
let config = SkeletorConfig::from_yaml_str(r#"
directories:
  src:
    main.rs: |
      fn main() {
          println!("Hello, world!");
      }
  tests:
    test.rs: "// Test content"
"#)?;

// Apply configuration to target directory
let result = apply_config(&config, Path::new("./my-project"), false, false)?;

println!("Created {} files and {} directories in {:?}",
    result.files_created, result.dirs_created, result.duration);
```

### Use Cases
- **MCP Servers**: Integrate with Model Context Protocol for AI-driven scaffolding
- **Web Services**: Create project templates via REST APIs
- **Build Tools**: Generate code structures in build pipelines
- **IDE Extensions**: Provide scaffolding capabilities in editors
- **Custom CLIs**: Build domain-specific scaffolding tools

See [`examples/library_demo.rs`](examples/library_demo.rs) for a complete example.

## Contributing
Contributions are welcome! Open an issue or submit a pull request.

### Development
```bash
# Clone and build
git clone https://github.com/jasonnathan/skeletor.git
cd skeletor
cargo build

# Run comprehensive test suite
cargo test

# Run tests with coverage analysis
cargo install cargo-llvm-cov  # Install coverage tool (once)
cargo llvm-cov --html         # Generate coverage report

# Code quality checks
cargo clippy -- -D warnings           # Lint with warnings as errors
cargo fmt --check                     # Check formatting
cargo check                           # Fast compilation check

# Test CLI commands locally
cargo run -- apply --dry-run --verbose    # Test apply with verbose output
cargo run -- snapshot --help              # Test command help
cargo run -- info -i 100k.yml             # Test info command
```

**Testing Guidelines:**
- All new features must include comprehensive unit tests
- Maintain test coverage above 80% (current: 82.51%)
- Use `cargo llvm-cov` to verify coverage before PR submission
- Test both CLI and library usage patterns

**Documentation Standards:**
- Update CHANGELOG.md for all user-facing changes
- Include examples for new CLI options
- Professional tone with lowercase prefixes (error:, info:, tip:)
- Preserve selective emojis for visual appeal (📁📄ℹ️✅⚠️)

### Releases
This project uses [cargo-release](https://github.com/crate-ci/cargo-release) for automated releases:

```bash
# Dry-run (see what would happen)
cargo release patch        # 0.2.22 → 0.2.23
cargo release minor        # 0.2.22 → 0.3.0  
cargo release major        # 0.2.22 → 1.0.0

# Actual release (maintainers only)
cargo release patch --execute
```

Releases automatically run tests and clippy checks to ensure code quality.

## License
This project is licensed under the MIT License.
Enjoy effortless scaffolding with Skeletor!

<p align="center">
  <sub>
    Proudly brought to you by 
    <a href="https://github.com/theGeekist" target="_blank">@theGeekist</a> and <a href="https://github.com/pipewrk" target="_blank">@pipewrk</a>
  </sub>
</p>

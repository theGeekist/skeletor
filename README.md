# Skeletor
![Build Status](https://github.com/jasonnathan/skeletor/actions/workflows/CI.yml/badge.svg)
[![codecov](https://codecov.io/gh/jasonnathan/skeletor/branch/main/graph/badge.svg)](https://codecov.io/gh/jasonnathan/skeletor)


A super optimized Rust scaffolding tool for generating files and directories from a YAML configuration. Skeletor is blazing fast and can create thousands of files and folders in milliseconds.

## Features

- **Generate Complex Directory Structures:**  
  Quickly create nested directories and files with predefined content.

- **Snapshot Mode:**  
  Generate a snapshot YAML (`.skeletorrc`) of an existing folder, including:
  - **Ignore Patterns:**  
    Use glob patterns or an existing `.gitignore` style file to exclude files/folders.
  - **Binary File Detection:**  
    Detect binary files and omit their content, while reporting them.
  - **Metadata & Stats:**  
    Automatically record timestamps (`created` and `updated`), generate statistics (number of files and directories), and attach generated comments. You can also add personal notes via the CLI.
- **Apply Mode:**  
  Build the file/directory structure based on the YAML configuration.

- **Dry-Run Mode:**  
  Preview operations without modifying the filesystem, available for both apply and snapshot commands.

- **Progress Visualization:**  
  A progress bar provides real-time feedback during operations.

## Installation

### Using Install Script
Use the provided install script to download and install the latest version of Skeletor.

```bash
curl -L https://github.com/jasonnathan/skeletor/releases/latest/download/install.sh | bash
```

### Homebrew (macOS)

```bash
brew tap jasonnathan/skeletor
brew install skeletor
```

### From Source

Ensure you have Rust and Cargo installed.

```bash
git clone https://github.com/jasonnathan/skeletor.git
cd skeletor
cargo install --path .
```

### Homebrew (macOS)

```bash
brew tap jasonnathan/skeletor
brew install skeletor
```

## Usage

Skeletor supports three subcommands: `apply`, `snapshot`, and `info`.

### Apply Command

Generate the file structure from a YAML configuration file (default: `.skeletorrc`).

```bash
skeletor apply [OPTIONS]
```

#### Options

- `-i, --input <FILE>`  
  Specify the input YAML configuration file (defaults to `.skeletorrc` if not provided).

- `-o, --overwrite`  
  Overwrite existing files.

- `-d, --dry-run`  
  Perform a trial run without making any changes.

### Snapshot Command

Create a snapshot of an existing folder into a YAML file (defaults to `.skeletorrc` if no output file is provided).

```bash
skeletor snapshot <FOLDER> [OPTIONS]
```

#### Options

- `-o, --output <FILE>`  
  Output file for the generated snapshot YAML (defaults to `.skeletorrc` if not provided).

- `--include-contents`  
  Include file contents for text files (binary file contents are omitted).

- `-I, --ignore <PATTERN_OR_FILE>`  
  Specify glob patterns or a file with .gitignore-style entries to ignore certain files or directories. This option can be used multiple times.

- `-n, --note <NOTE>`  
  Attach an optional user note to the snapshot.

- `-d, --dry-run`  
  Perform a trial run without writing changes to the filesystem.

The snapshot YAML now includes metadata:

- **created:** Timestamp when the snapshot was first generated.
- **updated:** Timestamp when the snapshot was last updated.
- **generated_comments:** Auto-generated details (including binary file detection).
- **notes:** Any user-supplied notes.
- **stats:** Statistics on the number of files and directories.
- **blacklist:** The active ignore patterns.

### Info Command

Display metadata and statistics from an existing snapshot file (default: `.skeletorrc`).

```bash
skeletor info [OPTIONS]
```

#### Options

- `-i, --input <FILE>`  
  Specify the snapshot YAML configuration file (defaults to `.skeletorrc` if not provided).

## Example YAML Configuration

Create a YAML file (e.g., `.skeletorrc`) like this:

```yaml
directories:
  src:
    main.rs: |
      fn main() {
          println!("Hello, Skeletor!");
      }
    lib.rs: ""
  Cargo.toml: |
    [package]
    name = "my_project"
    version = "0.1.0"
```

## Example Commands

- **Apply Configuration:**

  ```bash
  skeletor apply --overwrite
  ```

- **Dry-Run Apply:**

  ```bash
  skeletor apply --dry-run
  ```

- **Create a Snapshot with Ignore Patterns and a Note:**

  ```bash
  skeletor snapshot path/to/source --output snapshot.yaml --include-contents --ignore "*.tmp" --ignore .gitignore --note "Initial snapshot for my project"
  ```

- **Dry-Run Snapshot:**

  ```bash
  skeletor snapshot path/to/source --dry-run
  ```

- **Display Snapshot Information:**

  ```bash
  skeletor info
  ```

## Contributing

Contributions are welcome! Please submit a pull request or open an issue on GitHub.

## License

This project is licensed under the MIT License.

---

Enjoy building your projects faster and with more insight using Skeletor!

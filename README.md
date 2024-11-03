# Skeletor

A super optimized Rust scaffolding tool for generating files and directories from a YAML configuration. Skeletor is blazing fast and can create thousands of files and folders in milliseconds.

## Features

- Generate complex directory structures quickly.
- Supports nested directories and files with predefined content.
- Overwrite existing files with a command-line flag.
- Progress bar to visualize the creation process.

## Installation

### From Source

Ensure you have Rust and Cargo installed.

```bash
git clone https://github.com/yourusername/skeletor.git
cd skeletor
cargo install --path .
```

### Using Cargo

```bash
cargo install skeletor
```

*(Note: You need to publish your crate to [crates.io](https://crates.io/) for this to work.)*

### Homebrew (macOS)

```bash
brew tap jasonnathan/skeletor
brew install skeletor
```

### Chocolatey (Windows)

```bash
choco install skeletor
```

## Usage

Create a YAML configuration file (e.g., `.skeletorrc`):

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

Run Skeletor:

```bash
skeletor
```

### Command-Line Options

- `-i`, `--input`: Specify the input YAML configuration file.
- `-o`, `--overwrite`: Overwrite existing files.

## Contributing

Contributions are welcome! Please submit a pull request or open an issue on GitHub.

## License

This project is licensed under the MIT License.

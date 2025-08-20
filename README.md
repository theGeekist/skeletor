# Skeletor
![Build Status](https://github.com/jasonnathan/skeletor/actions/workflows/CI.yml/badge.svg)
[![codecov](https://codecov.io/gh/jasonnathan/skeletor/branch/main/graph/badge.svg)](https://codecov.io/gh/jasonnathan/skeletor)

<p align="center">
  <strong>A blazing-fast Rust scaffolding tool.</strong><br>
    Generate thousands of files and directories with file contents from a <code>YAML</code> configuration in milliseconds.
<br>
  <small>📸 Capture existing nested folders as YAML templates with snapshot mode.</small>
</p>


## 🚀 Usage
Skeletor simplifies **project scaffolding** with an easy-to-use YAML configuration.


### 🛠 Generate Files and Directories

#### With a `.skeletorrc` file
```bash
skeletor apply
```
#### Using a custom YAML config
```bash
skeletor apply -i custom.yml
```

### 📁 Example .skeletorrc Configuration
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
📁 This will **generate the entire folder structure** instantly!

**Preview Before Running**
```bash
skeletor apply --dry-run
```

## 🛠️ Installation

### Option 1: Install via Script (Linux/macOS)
```bash
curl -fsSL https://raw.githubusercontent.com/jasonnathan/skeletor/main/install.sh | bash
```
⚠️ **Tip:** Review the script before running to ensure security.  

### Option 2: Homebrew (macOS & Linux)
```bash
brew tap jasonnathan/skeletor
brew install skeletor
```
💡 Easiest method if Homebrew is installed. 

### Option 3: Cargo (Recommended for Rust Users)
```bash
cargo install skeletor
```
🔹 Installs directly from crates.io.

### Option 4: Build from Source 
```bash
git clone https://github.com/jasonnathan/skeletor.git
cd skeletor
cargo install --path .
```
🛠️ Rust & Cargo need to be installed on your system.  

## 🔥 Features
- ✅ Generate Nested Files & Directories Instantly
- ✅ Dry-Run Mode – Preview before applying
- ✅ Snapshot Mode – Convert an existing folder into YAML
- ✅ Ignore Patterns & Binary File Detection
- ✅ Metadata & Stats Included

## 📸 Snapshot Mode
Capture a YAML snapshot of an existing folder.

**Create a Snapshot**
```bash
skeletor snapshot .
```
**Ignore files and add a note**
```bash
skeletor snapshot -n "Removed .git folder"  -I .gitignore -I .git/ .
```


**Options**
- `-o custom.yml`  → Path to custom yaml file (defaults to `.skeletorrc`)
- `-I "*.log"` → Exclude files based on patterns (a path works too).  
- `-n "Initial snapshot"` → Add custom notes.

## 📊 Info Mode
Display metadata from a `.skeletorrc` file.

```bash
skeletor info
```

## 🤝 Contributing
Contributions are welcome! Open an issue or submit a pull request.

## 📜 License
This project is licensed under the MIT License.
✨ Enjoy effortless scaffolding with Skeletor! 🚀

<p align="center">
  <sub>
    Proudly brought to you by 
    <a href="https://github.com/theGeekist" target="_blank">@theGeekist</a> and <a href="https://github.com/pipewrk" target="_blank">@pipewrk</a>
  </sub>
</p>

# Skeletor
![Build Status](https://github.com/jasonnathan/skeletor/actions/workflows/CI.yml/badge.svg)
[![codecov](https://codecov.io/gh/jasonnathan/skeletor/branch/main/graph/badge.svg)](https://codecov.io/gh/jasonnathan/skeletor)

A **super optimized Rust scaffolding tool** for generating files and directories from a **YAML configuration**. Skeletor is **blazing fast**, capable of creating **thousands of files and folders in milliseconds**.

## 🚀 Usage

Skeletor is designed for **effortless project scaffolding** using a YAML configuration file.

### **Generate Files and Directories in One Command** 

- with a `.skeletorrc` file
```bash
skeletor apply
```
- with a `custom.yml` config file 
```bash
skeletor apply -i custom.yml
```

### **Example `.skeletorrc` Configuration**
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

### **Apply the Configuration**
```bash
skeletor apply
```
📁 This will **generate the entire folder structure** instantly!

### **Preview Without Modifications**
```bash
skeletor apply --dry-run
```

---

## 🛠️ Installation

### Option 1: Using Install Script (Linux/macOS)
For a quick installation:  
```bash
curl -fsSL https://raw.githubusercontent.com/jasonnathan/skeletor/main/install.sh | bash
```
⚠️ **Note:** Review the script before running to ensure security.  

---

### Option 2: Homebrew (macOS & Linux)
```bash
brew tap jasonnathan/skeletor
brew install skeletor
```
💡 If you already have Homebrew installed, this is the easiest method.  

### Option 3: Cargo (Recommended for Rust Users)
If you have Rust & Cargo installed, install directly from [crates.io](https://crates.io/crates/skeletor):  
```bash
cargo install skeletor
```
🔹 This ensures you always get the latest stable version.  

### Option 4: Build from Source
If you prefer to install manually:  
```bash
git clone https://github.com/jasonnathan/skeletor.git
cd skeletor
cargo install --path .
```
🛠️ Rust & Cargo need to be installed on your system.  

---

## 🔥 Features

- **Instantly Generate Nested Files & Directories**  
  Define project structures in YAML and apply them in one command.

- **Dry-Run Mode**  
  Preview directory creation without modifying the filesystem.

- **Snapshot Mode**  
  Convert an **existing folder** into a YAML `.skeletorrc` snapshot.

- **Ignore Patterns & Binary File Detection**  
  Supports `.gitignore`-style patterns, detects binary files, and omits content.

- **Metadata & Stats**  
  Includes timestamps, file counts, and user-defined notes.

---

## 📸 Snapshot Mode
Capture a **YAML snapshot** of an existing folder.

### **Create a Snapshot**
```bash
skeletor snapshot ./
```

### **Options**
- `-o custom.yml`  → Path to custom yaml file (defaults to `.skeletorrc`)
- `-i "*.log"` → Exclude files based on patterns (a path works too).  
- `-n "Initial snapshot"` → Add custom notes.

---

## 📊 Info Mode
Display metadata from a `.skeletorrc` file.

```bash
skeletor info
```

---

## 🤝 Contributing
Contributions are welcome! Open an issue or submit a pull request.

## 📜 License
This project is licensed under the MIT License.

---

Enjoy **effortless scaffolding** with Skeletor! 🚀
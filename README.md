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

### **Using Install Script**
```bash
curl -L https://raw.githubusercontent.com/jasonnathan/skeletor/main/install.sh | bash
```

### **Homebrew (macOS)**
```bash
brew tap jasonnathan/skeletor
brew install skeletor
```

### **From Source**
Ensure Rust & Cargo are installed.
```bash
git clone https://github.com/jasonnathan/skeletor.git
cd skeletor
cargo install --path .
```

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

- **Progress Visualization**  
  A real-time progress bar provides immediate feedback.

---

## 📸 Snapshot Mode
Capture a **YAML snapshot** of an existing folder.

### **Create a Snapshot**
```bash
skeletor snapshot my_project --output snapshot.yaml
```

### **Options**
- `--ignore "*.log"` → Exclude files based on patterns (a path works too).  
- `--note "Initial snapshot"` → Add custom notes.

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
//! Integration tests for the Skeletor CLI
//! 
//! These tests exercise the entire CLI pipeline including:
//! - CLI argument parsing
//! - File I/O operations  
//! - Error handling
//! - Output formatting

use std::fs;
use std::process::Command;
use tempfile::tempdir;

/// Test that help is displayed correctly
#[test]
fn test_cli_help() {
    let output = Command::new("cargo")
        .args(["run", "--", "--help"])
        .output()
        .expect("Failed to run skeletor help");
    
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Skeletor"));
    assert!(stdout.contains("apply"));
    assert!(stdout.contains("snapshot"));
    assert!(stdout.contains("info"));
}

/// Test apply subcommand with a simple configuration
#[test]
fn test_cli_apply_integration() {
    let temp_dir = tempdir().unwrap();
    let config_file = temp_dir.path().join("test.yml");
    
    // Create a simple test configuration
    let config_content = r#"
directories:
  hello_world:
    main.rs: |
      fn main() {
          println!("Hello, World!");
      }
    Cargo.toml: |
      [package]
      name = "hello_world"
      version = "0.1.0"
      edition = "2021"
  docs:
    README.md: "Hello World Documentation"
"#;
    fs::write(&config_file, config_content).unwrap();
    
    // Run apply from temp directory so files are created there
    let binary_path = std::env::current_dir().unwrap().join("target/debug/skeletor");
    
    // Ensure binary exists before trying to run it
    if !binary_path.exists() {
        panic!("Skeletor binary not found at {:?}. Run 'cargo build' first.", binary_path);
    }
    
    let output = Command::new(&binary_path)
        .args(["apply", config_file.to_str().unwrap()])
        .current_dir(&temp_dir)
        .output()
        .map_err(|e| format!("Failed to execute skeletor binary at {:?}: {}", binary_path, e))
        .expect("Failed to run skeletor apply");
    
    assert!(output.status.success(), "Apply command failed: {}", 
            String::from_utf8_lossy(&output.stderr));
    
    // Verify files were created
    assert!(temp_dir.path().join("hello_world/main.rs").exists());
    assert!(temp_dir.path().join("hello_world/Cargo.toml").exists());
    assert!(temp_dir.path().join("docs/README.md").exists());
    
    // Verify file contents
    let main_content = fs::read_to_string(temp_dir.path().join("hello_world/main.rs")).unwrap();
    assert!(main_content.contains("println!(\"Hello, World!\");"));
}

/// Test apply subcommand with dry run
#[test] 
fn test_cli_apply_dry_run() {
    let temp_dir = tempdir().unwrap();
    let config_file = temp_dir.path().join("test.yml");
    
    let config_content = r#"
directories:
  test_dir:
    file.txt: "content"
"#;
    fs::write(&config_file, config_content).unwrap();
    
    let output = Command::new("cargo")
        .args(["run", "--", "apply", "--dry-run", config_file.to_str().unwrap()])
        .output()
        .expect("Failed to run skeletor apply --dry-run");
    
    assert!(output.status.success(), "Apply dry-run failed. stderr: {}", 
            String::from_utf8_lossy(&output.stderr));
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Dry run"));
    
    // Verify no files were actually created
    assert!(!temp_dir.path().join("test_dir").exists());
}

/// Test snapshot subcommand
#[test]
fn test_cli_snapshot_integration() {
    let temp_dir = tempdir().unwrap();
    
    // Create a sample project structure
    fs::create_dir_all(temp_dir.path().join("src")).unwrap();
    fs::write(temp_dir.path().join("src/main.rs"), "fn main() {}").unwrap();
    fs::write(temp_dir.path().join("README.md"), "# Test Project").unwrap();
    
    let binary_path = std::env::current_dir().unwrap().join("target/debug/skeletor");
    let output = Command::new(binary_path)
        .args(["snapshot", "."])  // Snapshot current directory (the temp dir)
        .current_dir(&temp_dir)   // Run from temp directory
        .output()
        .expect("Failed to run skeletor snapshot");
    
    assert!(output.status.success(), "Snapshot command failed: {}",
            String::from_utf8_lossy(&output.stderr));
    
    // Check if snapshot was written to file in the temp directory
    let snapshot_file = temp_dir.path().join(".skeletorrc");
    assert!(snapshot_file.exists(), "Snapshot file should be created");
    
    let snapshot_content = fs::read_to_string(snapshot_file).unwrap();
    assert!(snapshot_content.contains("directories:"));
    assert!(snapshot_content.contains("src"));
    assert!(snapshot_content.contains("main.rs"));
}

/// Test info subcommand
#[test]
fn test_cli_info_integration() {
    let temp_dir = tempdir().unwrap();
    let config_file = temp_dir.path().join("config.yml");
    
    let config_content = r#"
created: "2024-01-01T00:00:00Z"
directories:
  src:
    main.rs: "fn main() {}"
stats:
  files: 1
  directories: 1
"#;
    fs::write(&config_file, config_content).unwrap();
    
    let output = Command::new("cargo")
        .args(["run", "--", "info", config_file.to_str().unwrap()])
        .output()
        .expect("Failed to run skeletor info");
    
    assert!(output.status.success(), "Info command failed: {}",
            String::from_utf8_lossy(&output.stderr));
    
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Created:"), "Expected 'Created:' in output: {}", stdout);
    assert!(stdout.contains("2024-01-01"));
}

/// Test error handling for missing config file
#[test]
fn test_cli_error_missing_config() {
    let output = Command::new("cargo")
        .args(["run", "--", "apply", "nonexistent.yml"])
        .output()
        .expect("Failed to run skeletor with missing config");
    
    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("error:") || stderr.contains("Error") || stderr.contains("not found"));
}

/// Test CLI version output
#[test] 
fn test_cli_version() {
    let output = Command::new("cargo")
        .args(["run", "--", "--version"])
        .output()
        .expect("Failed to run skeletor --version");
    
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    // Check that version output contains "Skeletor" and follows semver pattern
    assert!(stdout.contains("Skeletor"));
    // Verify it's a valid semver-like pattern (major.minor.patch)
    assert!(stdout.contains(env!("CARGO_PKG_VERSION")));
}

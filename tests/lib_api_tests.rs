use skeletor::{SkeletorConfig, apply_config};
use tempfile::tempdir;

#[test]
fn test_library_api() {
    let temp_dir = tempdir().unwrap();
    let target_path = temp_dir.path();
    
    // Test the new library API
    let config = SkeletorConfig::from_yaml_str(r#"
directories:
  src:
    main.rs: |
      fn main() {
          println!("Hello from library!");
      }
    lib.rs: ""
  tests:
    test.rs: "// Test content"
"#).unwrap();

    let result = apply_config(&config, target_path, false, false).unwrap();
    
    assert_eq!(result.files_created, 3);
    assert!(result.duration.as_micros() > 0);
    assert_eq!(result.tasks_total, 5); // 2 dirs + 3 files
    
    // Check that files were actually created
    assert!(target_path.join("src/main.rs").exists());
    assert!(target_path.join("src/lib.rs").exists());
    assert!(target_path.join("tests/test.rs").exists());
}

#[test]
fn test_library_api_dry_run() {
    let temp_dir = tempdir().unwrap();
    let target_path = temp_dir.path();
    
    let config = SkeletorConfig::from_yaml_str(r#"
directories:
  test_dir:
    test_file.txt: "content"
"#).unwrap();

    let result = apply_config(&config, target_path, false, true).unwrap();
    
    // In dry run, no files should be created
    assert_eq!(result.files_created, 0);
    assert_eq!(result.dirs_created, 0);
    assert_eq!(result.tasks_total, 2); // 1 dir + 1 file
    
    // Check that no files were actually created
    assert!(!target_path.join("test_dir").exists());
}
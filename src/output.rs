use std::path::PathBuf;
use std::time::Duration;
use termcolor::{StandardStream, ColorChoice, Color, ColorSpec, WriteColor};
use std::io::Write;
use crate::tasks::Task;

/// Simple result types for output module (without external dependencies)
#[derive(Debug, Clone)]
pub struct SimpleApplyResult {
    pub files_created: usize,
    pub dirs_created: usize,
    pub duration: Duration,
    pub tasks_total: usize,
    pub files_skipped: usize,
    pub skipped_files_list: Vec<String>,
    pub files_overwritten: usize,
    pub overwritten_files_list: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct SimpleSnapshotResult {
    pub files_processed: usize,
    pub dirs_processed: usize,
    pub duration: Duration,
    pub output_path: PathBuf,
    pub binary_files_excluded: usize,
    pub binary_files_list: Vec<String>,
}

impl SimpleApplyResult {
    #[allow(clippy::too_many_arguments)]    
    pub fn with_skipped_and_overwritten(
        files_created: usize, 
        dirs_created: usize, 
        duration: Duration, 
        tasks_total: usize,
        files_skipped: usize,
        skipped_files_list: Vec<String>,
        files_overwritten: usize,
        overwritten_files_list: Vec<String>
    ) -> Self {
        Self {
            files_created,
            dirs_created,
            duration,
            tasks_total,
            files_skipped,
            skipped_files_list,
            files_overwritten,
            overwritten_files_list,
        }
    }

    #[cfg(test)]
    pub fn new(files_created: usize, dirs_created: usize, duration: Duration, tasks_total: usize) -> Self {
        Self {
            files_created,
            dirs_created,
            duration,
            tasks_total,
            files_skipped: 0,
            skipped_files_list: Vec::new(),
            files_overwritten: 0,
            overwritten_files_list: Vec::new(),
        }
    }
}

/// Output formatting options
#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
pub enum OutputFormat {
    /// Plain text output
    Plain,
    /// Colored output with emoji and formatting
    Pretty,
}

/// Trait for reporting progress and results during operations
#[allow(dead_code)]
pub trait Reporter {
    /// Report the start of an operation
    fn operation_start(&self, operation: &str, details: &str);
    
    /// Report progress during an operation
    fn progress(&self, current: usize, total: usize, message: &str);
    
    /// Report a successful task completion
    fn task_success(&self, task: &Task);
    
    /// Report a task warning
    fn task_warning(&self, task: &Task, error: &str);
    
    /// Report a general warning
    fn warning(&self, message: &str);
    
    /// Report a general tip
    fn tip(&self, message: &str);
    
    /// Preview tasks in dry-run mode
    fn dry_run_preview(&self, tasks: &[Task]);
    
    /// Preview tasks in dry-run mode with verbose option
    fn dry_run_preview_verbose(&self, tasks: &[Task], verbose: bool);
    
    /// Preview tasks in dry-run mode with additional context (binary files, ignore patterns)
    fn dry_run_preview_comprehensive(&self, tasks: &[Task], verbose: bool, binary_files: &[String], ignore_patterns: &[String], verb: &str);
    
    /// Show operations that will be executed (verbose mode)
    fn verbose_operation_preview(&self, tasks: &[Task]);
    
    /// Report successful completion of apply operation with optional verbose output
    fn apply_complete(&self, result: &SimpleApplyResult, verbose: bool);
    
    /// Report completion of snapshot operation  
    fn snapshot_complete(&self, result: &SimpleSnapshotResult);
}

/// Default reporter with colored output
pub struct DefaultReporter {
    format: OutputFormat,
}

impl DefaultReporter {
    /// Create a new default reporter
    pub fn new() -> Self {
        Self::with_format(OutputFormat::Pretty)
    }
    
    /// Create a reporter with specific output format
    pub fn with_format(format: OutputFormat) -> Self {
        Self { format }
    }
    
    fn write_colored_inline(&self, text: &str, color: Option<Color>) {
        let mut stdout = StandardStream::stdout(ColorChoice::Auto);
        if let Some(c) = color {
            let _ = stdout.set_color(ColorSpec::new().set_fg(Some(c)).set_bold(true));
        }
        let _ = write!(stdout, "{}", text);
        let _ = stdout.reset();
    }

    fn summarize_tasks(tasks: &[Task]) -> (usize, usize) {
        tasks.iter().fold((0, 0), |(files, dirs), task| match task {
            Task::File(_, _) => (files + 1, dirs),
            Task::Dir(_) => (files, dirs + 1),
        })
    }

    fn print_task_list(&self, tasks: &[Task]) {
        for (i, task) in tasks.iter().enumerate() {
            match task {
                Task::File(path, _) => println!("  {}. 📄 {}", i + 1, path.display()),
                Task::Dir(path) => println!("  {}. 📁 {}", i + 1, path.display()),
            }
        }
    }

    fn print_task_preview(&self, tasks: &[Task], limit: usize, header: &str) {
        if !header.is_empty() {
            println!("{}", header);
        }
        for (i, task) in tasks.iter().take(limit).enumerate() {
            match task {
                Task::File(path, _) => println!("  {}. 📄 {}", i + 1, path.display()),
                Task::Dir(path) => println!("  {}. 📁 {}", i + 1, path.display()),
            }
        }
        if tasks.len() > limit {
            println!("  ... and {} more operations", tasks.len() - limit);
        }
    }

    fn print_string_list(
        &self,
        title: &str,
        items: &[String],
        verbose: bool,
        limit: usize,
        tip: Option<&str>,
    ) {
        if items.is_empty() {
            return;
        }

        println!("{}", title);
        if verbose || items.len() <= limit {
            for item in items {
                println!("  • {}", item);
            }
        } else {
            for item in items.iter().take(limit) {
                println!("  • {}", item);
            }
            println!("  ... and {} more", items.len() - limit);
            if let Some(tip) = tip {
                println!("tip: {}", tip);
            }
        }
    }
}

impl Reporter for DefaultReporter {
    fn operation_start(&self, operation: &str, details: &str) {
        match self.format {
            OutputFormat::Pretty => {
                self.write_colored_inline("start: ", Some(Color::Blue));
                println!("{}: {}", operation, details);
            },
            _ => println!("start: {}: {}", operation, details),
        }
    }
    
    fn progress(&self, current: usize, total: usize, message: &str) {
        match self.format {
            OutputFormat::Pretty => {
                self.write_colored_inline("progress: ", Some(Color::Yellow));
                println!("{}/{} - {}", current, total, message);
            },
            _ => println!("progress: {}/{} - {}", current, total, message),
        }
    }
    
    fn task_success(&self, task: &Task) {
        match self.format {
            OutputFormat::Pretty => {
                match task {
                    Task::Dir(path) => {
                        print!("📁 ");
                        self.write_colored_inline("Dir: ", Some(Color::Blue));
                        println!("{}", path.display());
                    },
                    Task::File(path, _) => {
                        print!("📄 ");
                        self.write_colored_inline("File: ", Some(Color::Green));
                        println!("{}", path.display());
                    },
                }
            },
            _ => {
                match task {
                    Task::Dir(path) => println!("✓ {}", path.display()),
                    Task::File(path, _) => println!("✓ {}", path.display()),
                }
            }
        }
    }
    
    fn task_warning(&self, task: &Task, error: &str) {
        match self.format {
            OutputFormat::Pretty => {
                self.write_colored_inline("warning: ", Some(Color::Yellow));
                match task {
                    Task::Dir(path) => println!("{}: {}", path.display(), error),
                    Task::File(path, _) => println!("{}: {}", path.display(), error),
                }
            },
            _ => {
                match task {
                    Task::Dir(path) => println!("warning: {}: {}", path.display(), error),
                    Task::File(path, _) => println!("warning: {}: {}", path.display(), error),
                }
            }
        }
    }
    
    fn warning(&self, message: &str) {
        match self.format {
            OutputFormat::Pretty => {
                self.write_colored_inline("warning: ", Some(Color::Yellow));
                println!("{}", message);
            },
            _ => println!("warning: {}", message),
        }
    }
    
    fn tip(&self, message: &str) {
        match self.format {
            OutputFormat::Pretty => {
                self.write_colored_inline("tip: ", Some(Color::Yellow));
                println!("{}", message);
            },
            _ => println!("tip: {}", message),
        }
    }
    
    fn dry_run_preview(&self, tasks: &[Task]) {
        self.dry_run_preview_verbose(tasks, false);
    }
    
    fn dry_run_preview_verbose(&self, tasks: &[Task], verbose: bool) {
        match self.format {
            OutputFormat::Pretty => {
                println!("Dry run enabled. Summary of planned operations:");
                
                let (file_count, dir_count) = Self::summarize_tasks(tasks);
                
                println!("  • {} files to be created", file_count);
                println!("  • {} directories to be created", dir_count);
                println!("  • Total: {} operations", tasks.len());
                
                // Show operations based on verbose flag
                if !tasks.is_empty() {
                    if verbose {
                        println!("\nComplete list of operations:");
                        self.print_task_list(tasks);
                    } else {
                        println!("\nSample of operations:");
                        self.print_task_preview(tasks, 5, "");
                        println!("\ntip: Use --verbose to see the complete operation list");
                    }
                }
                
                println!("\nDry run complete. No changes were made.");
            },
            _ => {
                println!("Dry run preview ({} tasks):", tasks.len());
                for task in tasks {
                    match task {
                        Task::Dir(path) => println!("  {}", path.display()),
                        Task::File(path, _) => println!("  {}", path.display()),
                    }
                }
            }
        }
    }
    
    fn dry_run_preview_comprehensive(&self, tasks: &[Task], verbose: bool, binary_files: &[String], ignore_patterns: &[String], verb: &str) {
        // Header
        println!("Dry run enabled.");
        println!();
        
        // Summary
        let (file_count, dir_count) = Self::summarize_tasks(tasks);
        
        println!("Summary of planned operations:");
        println!("  • {} files to be created", file_count);
        println!("  • {} directories to be created", dir_count);
        println!("  • Total: {} operations", tasks.len());
        println!();
        
        // Operations list
        if verbose && !tasks.is_empty() {
            println!("Complete list of operations:");
            self.print_task_list(tasks);
        } else if !tasks.is_empty() {
            self.print_task_preview(tasks, 3, "Operations preview (showing first 3):");
        }
        
        // Binary files
        if !binary_files.is_empty() {
            println!();
            self.print_string_list(
                &format!("Binary files that would be {}:", verb),
                binary_files,
                verbose,
                3,
                None,
            );
        }
        
        // Ignore patterns
        if !ignore_patterns.is_empty() {
            println!();
            self.print_string_list(
                "Ignore patterns that would be used:",
                ignore_patterns,
                verbose,
                3,
                None,
            );
        }
        
        // Footer with separator
        println!();
        println!("------------------------------------------");
        println!("Dry run complete. No changes were made.");
    }
    
    fn verbose_operation_preview(&self, tasks: &[Task]) {
        println!("Operations to be executed:");
        for (i, task) in tasks.iter().enumerate() {
            match task {
                Task::File(path, _) => {
                    println!("  {}. 📄 {}", i + 1, path.display());
                }
                Task::Dir(path) => {
                    println!("  {}. 📁 {}", i + 1, path.display());
                }
            }
        }
        println!();
    }
    
    fn apply_complete(&self, result: &SimpleApplyResult, verbose: bool) {
        match self.format {
            OutputFormat::Pretty => {
                // Show skipped files with helpful tip
                if result.files_skipped > 0 {
                    println!();
                    self.print_string_list(
                        "Files skipped (already exist):",
                        &result.skipped_files_list,
                        verbose,
                        3,
                        Some("Use --verbose to see all skipped files"),
                    );
                    println!();
                    self.write_colored_inline("tip: ", Some(Color::Yellow));
                    println!("Use --overwrite to update existing files");
                }
                
                // Show overwritten files
                if result.files_overwritten > 0 {
                    println!();
                    self.print_string_list(
                        "Files updated by --overwrite:",
                        &result.overwritten_files_list,
                        verbose,
                        3,
                        Some("Use --verbose to see all overwritten files"),
                    );
                }
                
                println!("------------------------------------------");
                let mut stdout = StandardStream::stdout(ColorChoice::Auto);
                print!("✅ Successfully generated {} files and {} directories in ", 
                       result.files_created, result.dirs_created);
                let _ = stdout.set_color(ColorSpec::new().set_fg(Some(Color::Cyan)).set_bold(true));
                let _ = write!(stdout, "{:.2}ms", result.duration.as_micros() as f64 / 1000.0);
                let _ = stdout.reset();
                println!();
            },
            _ => {
                println!("Success!");
                println!("Directories created: {}", result.dirs_created);
                println!("Files created: {}", result.files_created);
                if result.files_skipped > 0 {
                    println!("Files skipped: {}", result.files_skipped);
                }
                if result.files_overwritten > 0 {
                    println!("Files overwritten: {}", result.files_overwritten);
                }
                println!("Duration: {:.2}ms", result.duration.as_micros() as f64 / 1000.0);
                println!("Total operations: {}", result.tasks_total);
            }
        }
    }
    
    fn snapshot_complete(&self, result: &SimpleSnapshotResult) {
        match self.format {
            OutputFormat::Pretty => {
                self.write_colored_inline("Snapshot written to ", Some(Color::Green));
                println!("{:?}", result.output_path);
                
                // Show binary files excluded information if any
                self.print_string_list(
                    "Binary files excluded:",
                    &result.binary_files_list,
                    true,
                    3,
                    None,
                );
            },
            _ => {
                println!("Snapshot complete!");
                println!("Files processed: {}", result.files_processed);
                println!("Directories processed: {}", result.dirs_processed);
                println!("Duration: {:.2}ms", result.duration.as_micros() as f64 / 1000.0);
                println!("Output: {}", result.output_path.display());
                if result.binary_files_excluded > 0 {
                    println!("Binary files excluded: {}", result.binary_files_excluded);
                }
            }
        }
    }
}

/// Silent reporter that produces no output
#[allow(dead_code)]
pub struct SilentReporter;

impl Reporter for SilentReporter {
    fn operation_start(&self, _operation: &str, _details: &str) {}
    fn progress(&self, _current: usize, _total: usize, _message: &str) {}
    fn task_success(&self, _task: &Task) {}
    fn task_warning(&self, _task: &Task, _error: &str) {}
    fn warning(&self, _message: &str) {}
    fn tip(&self, _message: &str) {}
    fn dry_run_preview(&self, _tasks: &[Task]) {}
    fn dry_run_preview_verbose(&self, _tasks: &[Task], _verbose: bool) {}
    fn dry_run_preview_comprehensive(&self, _tasks: &[Task], _verbose: bool, _binary_files: &[String], _ignore_patterns: &[String], _verb: &str) {}
    fn verbose_operation_preview(&self, _tasks: &[Task]) {}
    fn apply_complete(&self, _result: &SimpleApplyResult, _verbose: bool) {}
    fn snapshot_complete(&self, _result: &SimpleSnapshotResult) {}
}

impl Default for DefaultReporter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::time::Duration;

    #[test]
    fn test_simple_apply_result_creation() {
        let simple_result = SimpleApplyResult::new(5, 3, Duration::from_millis(100), 8);
        assert_eq!(simple_result.files_created, 5);
        assert_eq!(simple_result.dirs_created, 3);
        assert_eq!(simple_result.duration, Duration::from_millis(100));
        assert_eq!(simple_result.tasks_total, 8);
    }

    #[test]
    fn test_default_reporter_creation() {
        let reporter = DefaultReporter::new();
        match reporter.format {
            OutputFormat::Pretty => {},
            _ => panic!("Expected Pretty format"),
        }
    }

    #[test]
    fn test_reporter_with_format() {
        let reporter = DefaultReporter::with_format(OutputFormat::Plain);
        match reporter.format {
            OutputFormat::Plain => {},
            _ => panic!("Expected Plain format"),
        }
    }

    #[test]
    fn test_silent_reporter_methods() {
        let reporter = SilentReporter;
        let task = Task::Dir("test".into());
        
        // These should not panic and should do nothing
        reporter.operation_start("test", "details");
        reporter.progress(1, 10, "message");
        reporter.task_success(&task);
        reporter.task_warning(&task, "warning");
        reporter.dry_run_preview(&[task]);
        
        let apply_result = SimpleApplyResult::new(1, 1, Duration::from_millis(50), 2);
        reporter.apply_complete(&apply_result, false);
        
        let snapshot_result = SimpleSnapshotResult {
            files_processed: 2,
            dirs_processed: 1,
            duration: Duration::from_millis(75),
            output_path: PathBuf::from("test.yml"),
            binary_files_excluded: 0,
            binary_files_list: vec![],
        };
        reporter.snapshot_complete(&snapshot_result);
    }

    #[test]
    fn test_default_reporter_methods() {
        let reporter = DefaultReporter::new();
        let task = Task::File("test.txt".into(), "content".to_string());
        
        // Test that these don't panic (output verification would need capturing stdout)
        reporter.operation_start("test operation", "details");
        reporter.progress(1, 10, "progress message");
        reporter.task_success(&task);
        reporter.task_warning(&task, "warning message");
        reporter.dry_run_preview(&[task]);
        
        let apply_result = SimpleApplyResult::new(3, 2, Duration::from_millis(150), 5);
        reporter.apply_complete(&apply_result, false);
        
        let snapshot_result = SimpleSnapshotResult {
            files_processed: 4,
            dirs_processed: 2,
            duration: Duration::from_millis(200),
            output_path: PathBuf::from("snapshot.yml"),
            binary_files_excluded: 1,
            binary_files_list: vec!["image.png".to_string()],
        };
        reporter.snapshot_complete(&snapshot_result);
    }

    #[test]
    fn test_plain_format_reporter() {
        let reporter = DefaultReporter::with_format(OutputFormat::Plain);
        let apply_result = SimpleApplyResult::new(2, 1, Duration::from_millis(75), 3);
        
        // Test that plain format doesn't panic
        reporter.apply_complete(&apply_result, false);
        
        let snapshot_result = SimpleSnapshotResult {
            files_processed: 5,
            dirs_processed: 3,
            duration: Duration::from_millis(125),
            output_path: PathBuf::from("plain.yml"),
            binary_files_excluded: 2,
            binary_files_list: vec!["image.png".to_string(), "video.mp4".to_string()],
        };
        reporter.snapshot_complete(&snapshot_result);
    }

    #[test]
    fn test_output_format_debug() {
        let format = OutputFormat::Pretty;
        let debug_str = format!("{:?}", format);
        assert!(debug_str.contains("Pretty"));
    }

    #[test]
    fn test_simple_results_debug() {
        let apply_result = SimpleApplyResult::new(1, 1, Duration::from_millis(50), 2);
        let debug_str = format!("{:?}", apply_result);
        assert!(debug_str.contains("files_created"));
        
        let snapshot_result = SimpleSnapshotResult {
            files_processed: 2,
            dirs_processed: 1,
            duration: Duration::from_millis(75),
            output_path: PathBuf::from("test.yml"),
            binary_files_excluded: 0,
            binary_files_list: vec![],
        };
        let debug_str = format!("{:?}", snapshot_result);
        assert!(debug_str.contains("files_processed"));
    }

    #[test]
    fn test_clone_functionality() {
        let apply_result = SimpleApplyResult::new(1, 1, Duration::from_millis(50), 2);
        let cloned = apply_result.clone();
        assert_eq!(cloned.files_created, apply_result.files_created);
        
        let snapshot_result = SimpleSnapshotResult {
            files_processed: 2,
            dirs_processed: 1,
            duration: Duration::from_millis(75),
            output_path: PathBuf::from("test.yml"),
            binary_files_excluded: 0,
            binary_files_list: vec![],
        };
        let cloned = snapshot_result.clone();
        assert_eq!(cloned.files_processed, snapshot_result.files_processed);
    }

    #[test]
    fn test_verbose_operation_preview() {
        let reporter = DefaultReporter::new();
        let tasks = vec![
            Task::Dir("test_output".into()),
            Task::File("test_output/hello.rs".into(), "fn main() {}".to_string()),
            Task::File("README.md".into(), "# Project".to_string()),
        ];
        
        // Test that it doesn't panic (output verification would need capturing stdout)
        reporter.verbose_operation_preview(&tasks);
    }

    #[test]
    fn test_verbose_operation_preview_empty() {
        let reporter = DefaultReporter::new();
        let tasks = vec![];
        
        // Test with empty task list
        reporter.verbose_operation_preview(&tasks);
    }

    #[test]
    fn test_dry_run_preview_comprehensive_verbose() {
        let reporter = DefaultReporter::new();
        let tasks = vec![
            Task::Dir("test_preview".into()),
            Task::File("test_preview/hello.rs".into(), "fn main() {}".to_string()),
        ];
        let binary_files = vec!["image.png".to_string(), "video.mp4".to_string()];
        let ignore_patterns = vec!["*.tmp".to_string(), "node_modules/".to_string()];
        
        // Test verbose mode
        reporter.dry_run_preview_comprehensive(&tasks, true, &binary_files, &ignore_patterns, "applied");
    }

    #[test]
    fn test_dry_run_preview_comprehensive_non_verbose() {
        let reporter = DefaultReporter::new();
        let tasks = vec![
            Task::Dir("test_nonverbose".into()),
            Task::File("test_nonverbose/hello.rs".into(), "fn main() {}".to_string()),
            Task::File("lib_file.rs".into(), "// lib".to_string()),
            Task::File("tests_file.rs".into(), "// tests".to_string()),
        ];
        let binary_files = vec!["img1.png".to_string(), "img2.jpg".to_string(), "img3.gif".to_string(), "img4.png".to_string()];
        let ignore_patterns = vec!["*.tmp".to_string(), "*.log".to_string(), "node_modules/".to_string(), "target/".to_string()];
        
        // Test non-verbose mode (should show first 3 + count)
        reporter.dry_run_preview_comprehensive(&tasks, false, &binary_files, &ignore_patterns, "captured");
    }

    #[test]
    fn test_dry_run_preview_comprehensive_empty_lists() {
        let reporter = DefaultReporter::new();
        let tasks = vec![Task::Dir("src".into())];
        let binary_files = vec![];
        let ignore_patterns = vec![];
        
        // Test with empty binary files and ignore patterns
        reporter.dry_run_preview_comprehensive(&tasks, true, &binary_files, &ignore_patterns, "processed");
    }

    #[test]
    fn test_silent_reporter_comprehensive() {
        let reporter = SilentReporter;
        let tasks = vec![Task::Dir("test".into())];
        let binary_files = vec!["test.bin".to_string()];
        let ignore_patterns = vec!["*.tmp".to_string()];
        
        // Test all methods on silent reporter
        reporter.dry_run_preview_verbose(&tasks, true);
        reporter.dry_run_preview_comprehensive(&tasks, true, &binary_files, &ignore_patterns, "processed");
        reporter.verbose_operation_preview(&tasks);
    }

    #[test]
    fn test_default_reporter_default_impl() {
        let reporter1 = DefaultReporter::default();
        let reporter2 = DefaultReporter::new();
        
        // Both should create Pretty format
        match (reporter1.format, reporter2.format) {
            (OutputFormat::Pretty, OutputFormat::Pretty) => {},
            _ => panic!("Default implementation doesn't match new()"),
        }
    }

    #[test]
    fn test_apply_complete_verbose_vs_non_verbose() {
        let reporter = DefaultReporter::new();
        
        // Test with verbose=true (should show all files)
        let apply_result = SimpleApplyResult::with_skipped_and_overwritten(
            5, 2, Duration::from_millis(100), 10,
            8, vec!["file1.txt".to_string(), "file2.txt".to_string(), "file3.txt".to_string(), "file4.txt".to_string()],
            3, vec!["over1.txt".to_string(), "over2.txt".to_string(), "over3.txt".to_string()]
        );
        reporter.apply_complete(&apply_result, true);
        
        // Test with verbose=false (should show limited files + tips)
        reporter.apply_complete(&apply_result, false);
    }

    #[test]
    fn test_apply_complete_no_skips_or_overwrites() {
        let reporter = DefaultReporter::new();
        
        // Test with no skipped or overwritten files
        let apply_result = SimpleApplyResult::with_skipped_and_overwritten(
            5, 2, Duration::from_millis(100), 7,
            0, vec![], 0, vec![]
        );
        reporter.apply_complete(&apply_result, false);
        reporter.apply_complete(&apply_result, true);
    }

    #[test]
    fn test_apply_complete_plain_format() {
        let reporter = DefaultReporter::with_format(OutputFormat::Plain);
        
        let apply_result = SimpleApplyResult::with_skipped_and_overwritten(
            3, 1, Duration::from_millis(75), 5,
            2, vec!["skip1.txt".to_string(), "skip2.txt".to_string()],
            1, vec!["over1.txt".to_string()]
        );
        reporter.apply_complete(&apply_result, false);
        reporter.apply_complete(&apply_result, true);
    }

    #[test]
    fn test_snapshot_complete_with_binary_files() {
        let reporter = DefaultReporter::new();
        
        let snapshot_result = SimpleSnapshotResult {
            files_processed: 10,
            dirs_processed: 3,
            duration: Duration::from_millis(150),
            output_path: PathBuf::from("snapshot.yml"),
            binary_files_excluded: 5,
            binary_files_list: vec![
                "image1.png".to_string(),
                "image2.jpg".to_string(),
                "binary.exe".to_string(),
                "video.mp4".to_string(),
                "data.bin".to_string(),
            ],
        };
        
        reporter.snapshot_complete(&snapshot_result);
    }

    #[test]
    fn test_snapshot_complete_no_binary_files() {
        let reporter = DefaultReporter::new();
        
        let snapshot_result = SimpleSnapshotResult {
            files_processed: 5,
            dirs_processed: 2,
            duration: Duration::from_millis(75),
            output_path: PathBuf::from("clean_snapshot.yml"),
            binary_files_excluded: 0,
            binary_files_list: vec![],
        };
        
        reporter.snapshot_complete(&snapshot_result);
    }

    #[test]
    fn test_snapshot_complete_plain_format() {
        let reporter = DefaultReporter::with_format(OutputFormat::Plain);
        
        let snapshot_result = SimpleSnapshotResult {
            files_processed: 7,
            dirs_processed: 2,
            duration: Duration::from_millis(100),
            output_path: PathBuf::from("plain_snapshot.yml"),
            binary_files_excluded: 2,
            binary_files_list: vec!["file1.bin".to_string(), "file2.exe".to_string()],
        };
        
        reporter.snapshot_complete(&snapshot_result);
    }

    #[test]
    fn test_with_skipped_and_overwritten_constructor() {
        let result = SimpleApplyResult::with_skipped_and_overwritten(
            10, 5, Duration::from_millis(200), 20,
            3, vec!["skip1".to_string(), "skip2".to_string(), "skip3".to_string()],
            2, vec!["over1".to_string(), "over2".to_string()]
        );
        
        assert_eq!(result.files_created, 10);
        assert_eq!(result.dirs_created, 5);
        assert_eq!(result.tasks_total, 20);
        assert_eq!(result.files_skipped, 3);
        assert_eq!(result.skipped_files_list.len(), 3);
        assert_eq!(result.files_overwritten, 2);
        assert_eq!(result.overwritten_files_list.len(), 2);
        assert_eq!(result.duration, Duration::from_millis(200));
    }

    #[test]
    fn test_color_output_functionality() {
        let reporter = DefaultReporter::new();
        
        // Test colored output methods (these methods have internal color logic)
        let task = Task::File("test.txt".into(), "content".to_string());
        reporter.task_success(&task);
        reporter.task_warning(&task, "test warning");
        reporter.operation_start("test", "test operation");
        reporter.progress(50, 100, "test.txt");
    }
}

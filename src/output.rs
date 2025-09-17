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
}

#[derive(Debug, Clone)]
pub struct SimpleSnapshotResult {
    pub files_processed: usize,
    pub dirs_processed: usize,
    pub duration: Duration,
    pub output_path: PathBuf,
    pub binary_files_excluded: usize,
}

impl SimpleApplyResult {
    pub fn from_apply_result(result: &crate::ApplyResult) -> Self {
        Self {
            files_created: result.files_created,
            dirs_created: result.dirs_created,
            duration: result.duration,
            tasks_total: result.tasks_total,
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
    /// JSON output for machine consumption
    Json,
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
    
    /// Preview tasks in dry-run mode
    fn dry_run_preview(&self, tasks: &[Task]);
    
    /// Report completion of apply operation
    fn apply_complete(&self, result: &SimpleApplyResult);
    
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
}

impl Reporter for DefaultReporter {
    fn operation_start(&self, operation: &str, details: &str) {
        match self.format {
            OutputFormat::Pretty => {
                print!("🚀 ");
                self.write_colored_inline("start: ", Some(Color::Blue));
                println!("{}: {}", operation, details);
            },
            _ => println!("start: {}: {}", operation, details),
        }
    }
    
    fn progress(&self, current: usize, total: usize, message: &str) {
        match self.format {
            OutputFormat::Pretty => {
                print!("⚡ ");
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
                print!("⚠️  ");
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
    
    fn dry_run_preview(&self, tasks: &[Task]) {
        match self.format {
            OutputFormat::Pretty => {
                print!("ℹ️  ");
                self.write_colored_inline("info: ", Some(Color::Cyan));
                println!("Dry run preview ({} tasks):", tasks.len());
                for task in tasks {
                    match task {
                        Task::Dir(path) => {
                            print!("  📁 ");
                            self.write_colored_inline("Dir: ", Some(Color::Blue));
                            println!("{}", path.display());
                        },
                        Task::File(path, _) => {
                            print!("  📄 ");
                            self.write_colored_inline("File: ", Some(Color::Green));
                            println!("{}", path.display());
                        },
                    }
                }
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
    
    fn apply_complete(&self, result: &SimpleApplyResult) {
        match self.format {
            OutputFormat::Pretty => {
                print!("✅ ");
                self.write_colored_inline("success: ", Some(Color::Green));
                println!("Apply complete!");
                print!("   📁 ");
                self.write_colored_inline("dirs: ", Some(Color::Blue));
                println!("{}", result.dirs_created);
                print!("   📄 ");
                self.write_colored_inline("files: ", Some(Color::Green));
                println!("{}", result.files_created);
                print!("   ⏱️  ");
                self.write_colored_inline("duration: ", Some(Color::Cyan));
                println!("{:.2}ms", result.duration.as_micros() as f64 / 1000.0);
                print!("   ℹ️  ");
                self.write_colored_inline("total operations: ", Some(Color::Magenta));
                println!("{}", result.tasks_total);
            },
            _ => {
                println!("Success!");
                println!("Directories created: {}", result.dirs_created);
                println!("Files created: {}", result.files_created);
                println!("Duration: {:.2}ms", result.duration.as_micros() as f64 / 1000.0);
                println!("Total operations: {}", result.tasks_total);
            }
        }
    }
    
    fn snapshot_complete(&self, result: &SimpleSnapshotResult) {
        match self.format {
            OutputFormat::Pretty => {
                print!("📸 ");
                self.write_colored_inline("snapshot: ", Some(Color::Green));
                println!("Complete!");
                print!("   📄 ");
                self.write_colored_inline("files: ", Some(Color::Green));
                println!("{}", result.files_processed);
                print!("   📁 ");
                self.write_colored_inline("dirs: ", Some(Color::Blue));
                println!("{}", result.dirs_processed);
                print!("   ⏱️  ");
                self.write_colored_inline("duration: ", Some(Color::Cyan));
                println!("{:.2}ms", result.duration.as_micros() as f64 / 1000.0);
                print!("   💾 ");
                self.write_colored_inline("output: ", Some(Color::Cyan));
                println!("Output: {}", result.output_path.display());
            },
            _ => {
                println!("Snapshot complete!");
                println!("Files processed: {}", result.files_processed);
                println!("Directories processed: {}", result.dirs_processed);
                println!("Duration: {:.2}ms", result.duration.as_micros() as f64 / 1000.0);
                println!("Output: {}", result.output_path.display());
            }
        }
    }
}

/// Silent reporter that produces no output
pub struct SilentReporter;

impl Reporter for SilentReporter {
    fn operation_start(&self, _operation: &str, _details: &str) {}
    fn progress(&self, _current: usize, _total: usize, _message: &str) {}
    fn task_success(&self, _task: &Task) {}
    fn task_warning(&self, _task: &Task, _error: &str) {}
    fn dry_run_preview(&self, _tasks: &[Task]) {}
    fn apply_complete(&self, _result: &SimpleApplyResult) {}
    fn snapshot_complete(&self, _result: &SimpleSnapshotResult) {}
}

impl Default for DefaultReporter {
    fn default() -> Self {
        Self::new()
    }
}

/// Utility functions for consistent output formatting
pub mod utils {
    use std::time::Duration;
    use termcolor::{StandardStream, ColorChoice, Color, ColorSpec, WriteColor};
    use std::io::Write;

    /// Print colored duration with consistent formatting across all commands
    pub fn print_colored_duration(prefix: &str, duration: Duration) {
        let mut stdout = StandardStream::stdout(ColorChoice::Auto);
        print!("{}", prefix);
        let _ = stdout.set_color(ColorSpec::new().set_fg(Some(Color::Cyan)).set_bold(true));
        let _ = write!(stdout, "{:.2}ms", duration.as_micros() as f64 / 1000.0);
        let _ = stdout.reset();
        println!();
    }

    /// Helper function for printing colored inline text (extracted from DefaultReporter)
    pub fn write_colored_inline(text: &str, color: Option<Color>) {
        let mut stdout = StandardStream::stdout(ColorChoice::Auto);
        if let Some(c) = color {
            let _ = stdout.set_color(ColorSpec::new().set_fg(Some(c)).set_bold(true));
        }
        let _ = write!(stdout, "{}", text);
        let _ = stdout.reset();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::time::Duration;

    #[test]
    fn test_simple_apply_result_creation() {
        let apply_result = crate::ApplyResult {
            files_created: 5,
            dirs_created: 3,
            duration: Duration::from_millis(100),
            tasks_total: 8,
        };
        
        let simple_result = SimpleApplyResult::from_apply_result(&apply_result);
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
        
        let apply_result = SimpleApplyResult {
            files_created: 1,
            dirs_created: 1,
            duration: Duration::from_millis(50),
            tasks_total: 2,
        };
        reporter.apply_complete(&apply_result);
        
        let snapshot_result = SimpleSnapshotResult {
            files_processed: 2,
            dirs_processed: 1,
            duration: Duration::from_millis(75),
            output_path: PathBuf::from("test.yml"),
            binary_files_excluded: 0,
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
        
        let apply_result = SimpleApplyResult {
            files_created: 3,
            dirs_created: 2,
            duration: Duration::from_millis(150),
            tasks_total: 5,
        };
        reporter.apply_complete(&apply_result);
        
        let snapshot_result = SimpleSnapshotResult {
            files_processed: 4,
            dirs_processed: 2,
            duration: Duration::from_millis(200),
            output_path: PathBuf::from("snapshot.yml"),
            binary_files_excluded: 1,
        };
        reporter.snapshot_complete(&snapshot_result);
    }

    #[test]
    fn test_plain_format_reporter() {
        let reporter = DefaultReporter::with_format(OutputFormat::Plain);
        let apply_result = SimpleApplyResult {
            files_created: 2,
            dirs_created: 1,
            duration: Duration::from_millis(75),
            tasks_total: 3,
        };
        
        // Test that plain format doesn't panic
        reporter.apply_complete(&apply_result);
        
        let snapshot_result = SimpleSnapshotResult {
            files_processed: 5,
            dirs_processed: 3,
            duration: Duration::from_millis(125),
            output_path: PathBuf::from("plain.yml"),
            binary_files_excluded: 2,
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
        let apply_result = SimpleApplyResult {
            files_created: 1,
            dirs_created: 1,
            duration: Duration::from_millis(50),
            tasks_total: 2,
        };
        let debug_str = format!("{:?}", apply_result);
        assert!(debug_str.contains("files_created"));
        
        let snapshot_result = SimpleSnapshotResult {
            files_processed: 2,
            dirs_processed: 1,
            duration: Duration::from_millis(75),
            output_path: PathBuf::from("test.yml"),
            binary_files_excluded: 0,
        };
        let debug_str = format!("{:?}", snapshot_result);
        assert!(debug_str.contains("files_processed"));
    }

    #[test]
    fn test_clone_functionality() {
        let apply_result = SimpleApplyResult {
            files_created: 1,
            dirs_created: 1,
            duration: Duration::from_millis(50),
            tasks_total: 2,
        };
        let cloned = apply_result.clone();
        assert_eq!(cloned.files_created, apply_result.files_created);
        
        let snapshot_result = SimpleSnapshotResult {
            files_processed: 2,
            dirs_processed: 1,
            duration: Duration::from_millis(75),
            output_path: PathBuf::from("test.yml"),
            binary_files_excluded: 0,
        };
        let cloned = snapshot_result.clone();
        assert_eq!(cloned.files_processed, snapshot_result.files_processed);
    }
}
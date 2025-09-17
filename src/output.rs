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
    
    fn write_colored(&self, text: &str, color: Option<Color>) {
        let mut stdout = StandardStream::stdout(ColorChoice::Auto);
        if let Some(c) = color {
            let _ = stdout.set_color(ColorSpec::new().set_fg(Some(c)));
        }
        let _ = write!(stdout, "{}", text);
        let _ = stdout.reset();
    }
}

impl Reporter for DefaultReporter {
    fn operation_start(&self, operation: &str, details: &str) {
        match self.format {
            OutputFormat::Pretty => {
                self.write_colored("🚀 ", Some(Color::Blue));
                println!("{}: {}", operation, details);
            },
            _ => println!("{}: {}", operation, details),
        }
    }
    
    fn progress(&self, current: usize, total: usize, message: &str) {
        match self.format {
            OutputFormat::Pretty => {
                self.write_colored("⚡ ", Some(Color::Yellow));
                println!("Progress: {}/{} - {}", current, total, message);
            },
            _ => println!("Progress: {}/{} - {}", current, total, message),
        }
    }
    
    fn task_success(&self, task: &Task) {
        match self.format {
            OutputFormat::Pretty => {
                match task {
                    Task::Dir(path) => {
                        self.write_colored("📁 ", Some(Color::Blue));
                        println!("{}", path.display());
                    },
                    Task::File(path, _) => {
                        self.write_colored("📄 ", Some(Color::Green));
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
                self.write_colored("⚠️  ", Some(Color::Yellow));
                match task {
                    Task::Dir(path) => println!("{}: {}", path.display(), error),
                    Task::File(path, _) => println!("{}: {}", path.display(), error),
                }
            },
            _ => {
                match task {
                    Task::Dir(path) => println!("! {}: {}", path.display(), error),
                    Task::File(path, _) => println!("! {}: {}", path.display(), error),
                }
            }
        }
    }
    
    fn dry_run_preview(&self, tasks: &[Task]) {
        match self.format {
            OutputFormat::Pretty => {
                self.write_colored("🔍 ", Some(Color::Cyan));
                println!("Dry run preview ({} tasks):", tasks.len());
                for task in tasks {
                    match task {
                        Task::Dir(path) => {
                            self.write_colored("  📁 ", Some(Color::Blue));
                            println!("{}", path.display());
                        },
                        Task::File(path, _) => {
                            self.write_colored("  📄 ", Some(Color::Green));
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
                self.write_colored("✅ ", Some(Color::Green));
                println!("Success!");
                self.write_colored("   📁 ", Some(Color::Blue));
                println!("Directories created: {}", result.dirs_created);
                self.write_colored("   📄 ", Some(Color::Green));
                println!("Files created: {}", result.files_created);
                self.write_colored("   ⚡ ", Some(Color::Yellow));
                println!("Duration: {:.2}ms", result.duration.as_micros() as f64 / 1000.0);
                self.write_colored("   🎯 ", Some(Color::Magenta));
                println!("Total operations: {}", result.tasks_total);
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
                self.write_colored("📸 ", Some(Color::Green));
                println!("Snapshot complete!");
                self.write_colored("   📄 ", Some(Color::Green));
                println!("Files processed: {}", result.files_processed);
                self.write_colored("   📁 ", Some(Color::Blue));
                println!("Directories processed: {}", result.dirs_processed);
                self.write_colored("   ⚡ ", Some(Color::Yellow));
                println!("Duration: {:.2}ms", result.duration.as_micros() as f64 / 1000.0);
                self.write_colored("   💾 ", Some(Color::Cyan));
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
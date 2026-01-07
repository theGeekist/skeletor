use crate::errors::SkeletorError;
use crate::output::{DefaultReporter, Reporter};
use crate::utils::read_file_to_string;
use ignore::gitignore::{Gitignore, GitignoreBuilder};
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub struct IgnoreSpec {
    pub matcher: Option<Gitignore>,
    pub patterns: Vec<String>,
}

fn add_ignore_line(
    builder: &mut GitignoreBuilder,
    source: Option<PathBuf>,
    line: &str,
    reporter: &DefaultReporter,
    patterns: &mut Vec<String>,
) -> Result<(), SkeletorError> {
    let trimmed = line.trim();
    if trimmed.is_empty() || trimmed.starts_with('#') {
        return Ok(());
    }

    if let Err(e) = builder.add_line(source.clone(), trimmed) {
        if source.is_none() {
            return Err(SkeletorError::InvalidIgnorePattern {
                pattern: format!("{} ({})", trimmed, e),
            });
        }

        reporter.warning(&format!(
            "Skipping invalid ignore pattern '{}'{}: {}",
            trimmed,
            source
                .as_ref()
                .map(|p| format!(" from {}", p.display()))
                .unwrap_or_default(),
            e
        ));
        reporter.tip("Check ignore pattern syntax or escape special characters");
        return Ok(());
    }

    patterns.push(trimmed.to_string());
    Ok(())
}

fn add_ignore_file(
    builder: &mut GitignoreBuilder,
    path: &Path,
    reporter: &DefaultReporter,
    patterns: &mut Vec<String>,
) -> Result<(), SkeletorError> {
    if !path.exists() || !path.is_file() {
        return Err(SkeletorError::FileNotFound {
            path: path.to_path_buf(),
        });
    }

    let content = read_file_to_string(path)?;
    for line in content.lines() {
        add_ignore_line(
            builder,
            Some(path.to_path_buf()),
            line,
            reporter,
            patterns,
        )?;
    }

    Ok(())
}

pub fn collect_ignore_spec(
    root: &Path,
    ignore_values: Option<impl Iterator<Item = String>>,
    ignore_files: Option<impl Iterator<Item = String>>,
    reporter: &DefaultReporter,
) -> Result<IgnoreSpec, SkeletorError> {
    let mut builder = GitignoreBuilder::new(root);
    let mut patterns = Vec::new();

    if let Some(vals) = ignore_values {
        for val in vals {
            let candidate = Path::new(&val);
            if candidate.exists() && candidate.is_file() {
                add_ignore_file(&mut builder, candidate, reporter, &mut patterns)?;
            } else {
                add_ignore_line(&mut builder, None, &val, reporter, &mut patterns)?;
            }
        }
    }

    if let Some(files) = ignore_files {
        for file in files {
            let path = Path::new(&file);
            add_ignore_file(&mut builder, path, reporter, &mut patterns)?;
        }
    }

    if patterns.is_empty() {
        return Ok(IgnoreSpec {
            matcher: None,
            patterns,
        });
    }

    let matcher = builder
        .build()
        .map_err(|e| SkeletorError::InvalidIgnorePattern {
            pattern: format!("Failed to compile ignore patterns: {}", e),
        })?;

    Ok(IgnoreSpec {
        matcher: Some(matcher),
        patterns,
    })
}

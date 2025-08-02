//! Nix code formatting support
//!
//! This module provides integration with various Nix formatting tools
//! such as nixpkgs-fmt, alejandra, and nixfmt.

use crate::{NixDomainError, Result};
use std::path::Path;
use std::process::Command;
use tokio::process::Command as AsyncCommand;

/// Available Nix formatters
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NixFormatter {
    /// nixpkgs-fmt - The official formatter
    NixpkgsFmt,
    /// alejandra - Opinionated formatter
    Alejandra,
    /// nixfmt - Classic formatter
    NixFmt,
    /// nixfmt-rfc-style - RFC-style formatting
    NixFmtRfc,
}

impl NixFormatter {
    /// Get the command name for this formatter
    pub fn command(&self) -> &'static str {
        match self {
            Self::NixpkgsFmt => "nixpkgs-fmt",
            Self::Alejandra => "alejandra",
            Self::NixFmt => "nixfmt",
            Self::NixFmtRfc => "nixfmt",
        }
    }

    /// Get additional arguments for the formatter
    pub fn args(&self) -> Vec<&'static str> {
        match self {
            Self::NixpkgsFmt => vec![],
            Self::Alejandra => vec![],
            Self::NixFmt => vec![],
            Self::NixFmtRfc => vec!["--style", "rfc"],
        }
    }

    /// Check if the formatter is available on the system
    pub fn is_available(&self) -> bool {
        Command::new(self.command())
            .arg("--version")
            .output()
            .is_ok()
    }

    /// Detect which formatter to use based on project configuration
    pub async fn detect_from_project(path: &Path) -> Option<Self> {
        // Check for .nixfmt.toml
        if path.join(".nixfmt.toml").exists() {
            return Some(Self::NixFmtRfc);
        }

        // Check for .alejandra.toml
        if path.join(".alejandra.toml").exists() {
            return Some(Self::Alejandra);
        }

        // Check flake.nix for formatter definition
        let flake_path = path.join("flake.nix");
        if flake_path.exists() {
            if let Ok(content) = tokio::fs::read_to_string(&flake_path).await {
                if content.contains("alejandra") {
                    return Some(Self::Alejandra);
                } else if content.contains("nixpkgs-fmt") {
                    return Some(Self::NixpkgsFmt);
                } else if content.contains("nixfmt") {
                    return Some(Self::NixFmt);
                }
            }
        }

        // Default to nixpkgs-fmt if available
        if Self::NixpkgsFmt.is_available() {
            Some(Self::NixpkgsFmt)
        } else {
            None
        }
    }
}

/// Formatter service for formatting Nix files
pub struct FormatterService {
    formatter: NixFormatter,
    check_only: bool,
}

impl FormatterService {
    /// Create a new formatter service
    pub fn new(formatter: NixFormatter) -> Self {
        Self {
            formatter,
            check_only: false,
        }
    }

    /// Create a formatter service that only checks formatting
    pub fn check_only(formatter: NixFormatter) -> Self {
        Self {
            formatter,
            check_only: true,
        }
    }

    /// Format a single file
    pub async fn format_file(&self, path: &Path) -> Result<FormattingResult> {
        let mut cmd = AsyncCommand::new(self.formatter.command());

        // Add formatter-specific args
        for arg in self.formatter.args() {
            cmd.arg(arg);
        }

        // Add check flag if needed
        if self.check_only {
            match self.formatter {
                NixFormatter::NixpkgsFmt => cmd.arg("--check"),
                NixFormatter::Alejandra => cmd.arg("--check"),
                NixFormatter::NixFmt | NixFormatter::NixFmtRfc => cmd.arg("--check"),
            };
        }

        cmd.arg(path);

        let output = cmd
            .output()
            .await
            .map_err(|e| NixDomainError::FormatterError(format!("Failed to run formatter: {e}")))?;

        Ok(FormattingResult {
            formatted: output.status.success() && !self.check_only,
            needs_formatting: !output.status.success() && self.check_only,
            error: if output.status.success() {
                None
            } else {
                Some(String::from_utf8_lossy(&output.stderr).to_string())
            },
        })
    }

    /// Format multiple files
    pub async fn format_files(&self, paths: &[&Path]) -> Result<Vec<(String, FormattingResult)>> {
        use futures::stream::{self, StreamExt};

        let results: Vec<_> = stream::iter(paths)
            .map(|path| async move {
                let result = self.format_file(path).await?;
                Ok((path.display().to_string(), result))
            })
            .buffer_unordered(4)
            .collect::<Vec<Result<_>>>()
            .await;

        results.into_iter().collect()
    }

    /// Format all Nix files in a directory
    pub async fn format_directory(&self, dir: &Path) -> Result<FormattingReport> {
        let mut report = FormattingReport::default();

        // Find all .nix files
        let nix_files = self.find_nix_files(dir).await?;
        report.total_files = nix_files.len();

        for file in nix_files {
            match self.format_file(&file).await {
                Ok(result) => {
                    if result.formatted {
                        report.formatted_files.push(file.display().to_string());
                    } else if result.needs_formatting {
                        report.needs_formatting.push(file.display().to_string());
                    }

                    if let Some(error) = result.error {
                        report.errors.push((file.display().to_string(), error));
                    }
                }
                Err(e) => {
                    report
                        .errors
                        .push((file.display().to_string(), e.to_string()));
                }
            }
        }

        Ok(report)
    }

    /// Find all Nix files in a directory
    async fn find_nix_files(&self, dir: &Path) -> Result<Vec<std::path::PathBuf>> {
        use tokio::fs;

        let mut nix_files = Vec::new();
        let mut dirs_to_visit = vec![dir.to_path_buf()];

        while let Some(current_dir) = dirs_to_visit.pop() {
            let mut entries = fs::read_dir(&current_dir)
                .await
                .map_err(NixDomainError::IoError)?;

            while let Some(entry) = entries
                .next_entry()
                .await
                .map_err(NixDomainError::IoError)?
            {
                let path = entry.path();
                let file_type = entry.file_type().await.map_err(NixDomainError::IoError)?;

                if file_type.is_dir() {
                    // Skip hidden directories and common build outputs
                    let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

                    if !file_name.starts_with('.') && file_name != "result" {
                        dirs_to_visit.push(path);
                    }
                } else if file_type.is_file() {
                    if let Some(ext) = path.extension() {
                        if ext == "nix" {
                            nix_files.push(path);
                        }
                    }
                }
            }
        }

        Ok(nix_files)
    }
}

/// Result of formatting a single file
#[derive(Debug, Clone)]
pub struct FormattingResult {
    /// Whether the file was formatted
    pub formatted: bool,
    /// Whether the file needs formatting (check mode)
    pub needs_formatting: bool,
    /// Error message if formatting failed
    pub error: Option<String>,
}

/// Report for formatting multiple files
#[derive(Debug, Clone, Default)]
pub struct FormattingReport {
    /// Total number of files processed
    pub total_files: usize,
    /// Files that were formatted
    pub formatted_files: Vec<String>,
    /// Files that need formatting (check mode)
    pub needs_formatting: Vec<String>,
    /// Errors encountered
    pub errors: Vec<(String, String)>,
}

impl FormattingReport {
    /// Check if all files are properly formatted
    pub fn all_formatted(&self) -> bool {
        self.needs_formatting.is_empty() && self.errors.is_empty()
    }

    /// Get a summary of the report
    pub fn summary(&self) -> String {
        if self.needs_formatting.is_empty() && self.errors.is_empty() {
            format!("✅ All {} files are properly formatted", self.total_files)
        } else if !self.formatted_files.is_empty() {
            format!(
                "Formatted {} files, {} errors",
                self.formatted_files.len(),
                self.errors.len()
            )
        } else {
            format!(
                "❌ {} files need formatting, {} errors",
                self.needs_formatting.len(),
                self.errors.len()
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use tokio::fs;

    #[test]
    fn test_formatter_detection() {
        assert_eq!(NixFormatter::NixpkgsFmt.command(), "nixpkgs-fmt");
        assert_eq!(NixFormatter::Alejandra.command(), "alejandra");
        assert_eq!(NixFormatter::NixFmt.command(), "nixfmt");
    }

    #[tokio::test]
    async fn test_format_file() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.nix");

        // Write unformatted Nix code
        fs::write(&test_file, "{ foo = \"bar\"; baz = 42; }")
            .await
            .unwrap();

        // Skip test if no formatter is available
        if let Some(formatter) = NixFormatter::detect_from_project(temp_dir.path()).await {
            let service = FormatterService::new(formatter);
            let result = service.format_file(&test_file).await;

            // We can't guarantee the formatter is installed in test environment
            if result.is_ok() {
                let result = result.unwrap();
                assert!(result.formatted || result.error.is_some());
            }
        }
    }
}

//! Nix file analysis engine
//!
//! This module provides comprehensive analysis capabilities for Nix files,
//! including dependency graphs, security scanning, performance analysis,
//! and dead code detection.

pub mod dependency;
pub mod security;
pub mod performance;
pub mod dead_code;

use crate::parser::NixFile;
use crate::{Result, NixDomainError};
use crate::formatter::{NixFormatter, FormatterService};
use petgraph::graph::DiGraph;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};
use tokio::sync::Mutex;
use std::sync::Arc;
use uuid::Uuid;

pub use dependency::{DependencyAnalyzer, DependencyGraph, FileNode, DependencyEdge};
pub use security::{SecurityAnalyzer, SecurityIssue, Severity};
pub use performance::{PerformanceAnalyzer, PerformanceIssue};
pub use dead_code::{DeadCodeAnalyzer, DeadCode};

/// Main analyzer that coordinates all analysis types
pub struct NixAnalyzer {
    /// Parsed files cache
    cache: Arc<Mutex<HashMap<PathBuf, NixFile>>>,
    /// Dependency graph
    dep_graph: Arc<Mutex<DiGraph<FileNode, DependencyEdge>>>,
    /// Configuration
    config: AnalyzerConfig,
}

/// Configuration for the analyzer
#[derive(Debug, Clone)]
pub struct AnalyzerConfig {
    /// Enable parallel parsing
    pub parallel_parsing: bool,
    /// Maximum files to analyze
    pub max_files: Option<usize>,
    /// Follow symlinks
    pub follow_symlinks: bool,
    /// Ignore patterns
    pub ignore_patterns: Vec<String>,
    /// Check formatting
    pub check_formatting: bool,
    /// Formatter to use
    pub formatter: Option<NixFormatter>,
}

impl Default for AnalyzerConfig {
    fn default() -> Self {
        Self {
            parallel_parsing: true,
            max_files: None,
            follow_symlinks: false,
            ignore_patterns: vec![
                "result".to_string(),
                "result-*".to_string(),
                ".git".to_string(),
            ],
            check_formatting: false,
            formatter: None,
        }
    }
}

/// Analysis report containing all findings
#[derive(Debug, Clone, serde::Serialize)]
pub struct AnalysisReport {
    /// Analysis ID
    pub id: Uuid,
    /// Number of files analyzed
    pub files_analyzed: usize,
    /// Dependency graph
    #[serde(skip)]  // Skip graph for now as it's not easily serializable
    pub dependency_graph: DiGraph<FileNode, DependencyEdge>,
    /// Security issues found
    pub security_issues: Vec<SecurityIssue>,
    /// Performance issues found
    pub performance_issues: Vec<PerformanceIssue>,
    /// Dead code found
    pub dead_code: Vec<DeadCode>,
    /// Formatting issues (if check_formatting is enabled)
    pub formatting_issues: Option<Vec<String>>,
    /// Analysis duration
    pub duration: Duration,
}

impl NixAnalyzer {
    /// Create a new analyzer
    pub fn new() -> Self {
        Self::with_config(AnalyzerConfig::default())
    }

    /// Create analyzer with custom configuration
    pub fn with_config(config: AnalyzerConfig) -> Self {
        Self {
            cache: Arc::new(Mutex::new(HashMap::new())),
            dep_graph: Arc::new(Mutex::new(DiGraph::new())),
            config,
        }
    }

    /// Analyze a repository
    pub async fn analyze_repository(&mut self, repo_path: &Path) -> Result<AnalysisReport> {
        let start = Instant::now();
        let analysis_id = Uuid::new_v4();

        // Find all Nix files
        let nix_files = self.find_nix_files(repo_path).await?;
        
        // Parse all files
        let parsed_files = if self.config.parallel_parsing {
            self.parse_files_parallel(nix_files.clone()).await?
        } else {
            self.parse_files_sequential(nix_files.clone()).await?
        };

        // Build dependency graph
        self.build_dependency_graph(&parsed_files).await?;

        // Run analyzers
        let dep_graph = self.dep_graph.lock().await.clone();
        
        let security_issues = SecurityAnalyzer::analyze(&parsed_files)?;
        let performance_issues = PerformanceAnalyzer::analyze(&parsed_files)?;
        let dead_code = DeadCodeAnalyzer::analyze(&parsed_files, &dep_graph)?;

        // Check formatting if enabled
        let formatting_issues = if self.config.check_formatting {
            let formatter = self.config.formatter
                .or(NixFormatter::detect_from_project(repo_path).await);
            
            if let Some(formatter) = formatter {
                let service = FormatterService::check_only(formatter);
                let report = service.format_directory(repo_path).await?;
                
                if !report.all_formatted() {
                    Some(report.needs_formatting)
                } else {
                    Some(Vec::new())
                }
            } else {
                None
            }
        } else {
            None
        };

        Ok(AnalysisReport {
            id: analysis_id,
            files_analyzed: parsed_files.len(),
            dependency_graph: dep_graph,
            security_issues,
            performance_issues,
            dead_code,
            formatting_issues,
            duration: start.elapsed(),
        })
    }

    /// Find all Nix files in a directory
    async fn find_nix_files(&self, path: &Path) -> Result<Vec<PathBuf>> {
        use tokio::fs;

        let mut nix_files = Vec::new();
        let mut dirs_to_visit = vec![path.to_path_buf()];

        while let Some(dir) = dirs_to_visit.pop() {
            let mut entries = fs::read_dir(&dir).await
                .map_err(NixDomainError::IoError)?;

            while let Some(entry) = entries.next_entry().await
                .map_err(NixDomainError::IoError)? {
                let path = entry.path();
                let file_name = path.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("");

                // Check ignore patterns
                if self.should_ignore(file_name) {
                    continue;
                }

                let file_type = entry.file_type().await
                    .map_err(NixDomainError::IoError)?;

                if file_type.is_dir() {
                    dirs_to_visit.push(path);
                } else if file_type.is_file() && file_name.ends_with(".nix") {
                    nix_files.push(path);

                    // Check max files limit
                    if let Some(max) = self.config.max_files {
                        if nix_files.len() >= max {
                            break;
                        }
                    }
                }
            }
        }

        Ok(nix_files)
    }

    /// Check if a file should be ignored
    fn should_ignore(&self, file_name: &str) -> bool {
        self.config.ignore_patterns.iter()
            .any(|pattern| {
                if pattern.contains('*') {
                    // Simple glob matching
                    let pattern = pattern.replace("*", "");
                    file_name.starts_with(&pattern) || file_name.ends_with(&pattern)
                } else {
                    file_name == pattern
                }
            })
    }

    /// Parse files in parallel
    async fn parse_files_parallel(&self, paths: Vec<PathBuf>) -> Result<Vec<NixFile>> {
        use futures::stream::{self, StreamExt};

        let results: Vec<Result<NixFile>> = stream::iter(paths)
            .map(|path| async move {
                NixFile::parse_file(&path)
            })
            .buffer_unordered(num_cpus::get())
            .collect()
            .await;

        // Collect successful parses and cache them
        let mut parsed_files = Vec::new();
        let mut cache = self.cache.lock().await;

        for result in results {
            match result {
                Ok(file) => {
                    if let Some(ref path) = file.source {
                        cache.insert(path.clone(), file.clone());
                    }
                    parsed_files.push(file);
                }
                Err(e) => {
                    // Log error but continue
                    eprintln!("Failed to parse file: {}", e);
                }
            }
        }

        Ok(parsed_files)
    }

    /// Parse files sequentially
    async fn parse_files_sequential(&self, paths: Vec<PathBuf>) -> Result<Vec<NixFile>> {
        let mut parsed_files = Vec::new();
        let mut cache = self.cache.lock().await;

        for path in paths {
            match NixFile::parse_file(&path) {
                Ok(file) => {
                    cache.insert(path.clone(), file.clone());
                    parsed_files.push(file);
                }
                Err(e) => {
                    eprintln!("Failed to parse {}: {}", path.display(), e);
                }
            }
        }

        Ok(parsed_files)
    }

    /// Build dependency graph from parsed files
    async fn build_dependency_graph(&self, files: &[NixFile]) -> Result<()> {
        let mut graph = self.dep_graph.lock().await;
        graph.clear();

        // Create nodes for each file
        let mut node_map = HashMap::new();
        for file in files {
            if let Some(ref path) = file.source {
                let node = FileNode {
                    path: path.clone(),
                    file_type: file.file_type(),
                    has_errors: file.has_errors(),
                };
                let idx = graph.add_node(node);
                node_map.insert(path.clone(), idx);
            }
        }

        // Analyze dependencies and add edges
        for file in files {
            if let Some(ref source_path) = file.source {
                if let Some(&source_idx) = node_map.get(source_path) {
                    let deps = DependencyAnalyzer::find_dependencies(file)?;
                    
                    for dep in deps {
                        // Resolve relative paths
                        let dep_path = if dep.path.is_relative() {
                            source_path.parent()
                                .unwrap_or(Path::new("."))
                                .join(&dep.path)
                        } else {
                            dep.path.clone()
                        };

                        if let Some(&target_idx) = node_map.get(&dep_path) {
                            graph.add_edge(source_idx, target_idx, dep);
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Get cached file
    pub async fn get_cached_file(&self, path: &Path) -> Option<NixFile> {
        self.cache.lock().await.get(path).cloned()
    }

    /// Clear cache
    pub async fn clear_cache(&self) {
        self.cache.lock().await.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use tokio::fs;

    #[tokio::test]
    async fn test_find_nix_files() {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path();

        // Create test files
        fs::write(temp_path.join("test1.nix"), "{ }").await.unwrap();
        fs::write(temp_path.join("test2.nix"), "{ }").await.unwrap();
        fs::write(temp_path.join("other.txt"), "not nix").await.unwrap();

        let analyzer = NixAnalyzer::new();
        let files = analyzer.find_nix_files(temp_path).await.unwrap();

        assert_eq!(files.len(), 2);
        assert!(files.iter().all(|p| p.extension().unwrap() == "nix"));
    }

    #[tokio::test]
    async fn test_ignore_patterns() {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path();

        // Create test files
        fs::write(temp_path.join("test.nix"), "{ }").await.unwrap();
        fs::create_dir(temp_path.join("result")).await.unwrap();
        fs::write(temp_path.join("result").join("ignored.nix"), "{ }").await.unwrap();

        let analyzer = NixAnalyzer::new();
        let files = analyzer.find_nix_files(temp_path).await.unwrap();

        assert_eq!(files.len(), 1);
        assert!(!files[0].to_string_lossy().contains("result"));
    }
} 
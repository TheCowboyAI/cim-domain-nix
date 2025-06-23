//! Git-Nix analysis functionality
//!
//! This module provides analysis capabilities for Nix files in Git repositories,
//! including flake.lock history tracking and dependency change analysis.

use crate::{Result, NixDomainError};
use crate::git_integration::{FlakeLockCommit, DependencyChanges, GitFlakeInput};
use cim_domain_git::value_objects::{CommitHash, AuthorInfo};
use chrono::{DateTime, Utc};
use std::path::Path;
use std::collections::HashMap;
use serde::{Serialize, Deserialize};

/// Analyzer for Git-Nix integration
pub struct GitNixAnalyzer {
    // Future: Add caching and configuration options
}

impl GitNixAnalyzer {
    /// Create a new analyzer
    pub fn new() -> Self {
        Self {}
    }

    /// Get the history of flake.lock changes
    pub async fn get_flake_lock_history(
        &self,
        repo_path: &Path,
        limit: Option<usize>,
    ) -> Result<Vec<FlakeLockCommit>> {
        let mut commits = Vec::new();

        // Get Git log for flake.lock
        let mut cmd = tokio::process::Command::new("git");
        cmd.current_dir(repo_path)
            .args(&[
                "log",
                "--format=%H|%at|%an|%ae|%s",
                "--follow",
                "--",
                "flake.lock",
            ]);

        if let Some(limit) = limit {
            cmd.arg(format!("-{}", limit));
        }

        let output = cmd.output().await
            .map_err(|e| NixDomainError::CommandError(format!("Failed to get git log: {}", e)))?;

        if !output.status.success() {
            return Err(NixDomainError::CommandError(
                String::from_utf8_lossy(&output.stderr).to_string()
            ));
        }

        let log_output = String::from_utf8_lossy(&output.stdout);
        
        for line in log_output.lines() {
            if line.is_empty() {
                continue;
            }

            let parts: Vec<&str> = line.split('|').collect();
            if parts.len() < 5 {
                continue;
            }

            let commit_hash = CommitHash::new(parts[0])
                .map_err(|e| NixDomainError::ParseError(e.to_string()))?;
            
            let timestamp = parts[1].parse::<i64>()
                .ok()
                .and_then(|ts| DateTime::from_timestamp(ts, 0))
                .ok_or_else(|| NixDomainError::ParseError("Invalid timestamp".to_string()))?;

            let author = AuthorInfo::new(parts[2], parts[3]);
            let message = parts[4..].join("|");

            // Get the flake.lock content at this commit
            let lock_content = self.get_file_at_commit(
                repo_path,
                &commit_hash,
                "flake.lock"
            ).await?;

            let lock_json: serde_json::Value = serde_json::from_str(&lock_content)
                .map_err(|e| NixDomainError::ParseError(format!("Invalid flake.lock JSON: {}", e)))?;

            commits.push(FlakeLockCommit {
                commit: commit_hash,
                timestamp,
                message,
                author,
                lock_content: lock_json,
            });
        }

        Ok(commits)
    }

    /// Analyze dependency changes between two commits
    pub async fn analyze_dependency_changes(
        &self,
        repo_path: &Path,
        from_commit: &CommitHash,
        to_commit: &CommitHash,
    ) -> Result<DependencyChanges> {
        // Get flake.lock content at both commits
        let from_content = self.get_file_at_commit(repo_path, from_commit, "flake.lock").await?;
        let to_content = self.get_file_at_commit(repo_path, to_commit, "flake.lock").await?;

        let from_json: serde_json::Value = serde_json::from_str(&from_content)
            .map_err(|e| NixDomainError::ParseError(format!("Invalid from flake.lock: {}", e)))?;
        let to_json: serde_json::Value = serde_json::from_str(&to_content)
            .map_err(|e| NixDomainError::ParseError(format!("Invalid to flake.lock: {}", e)))?;

        // Parse inputs from both versions
        let from_inputs = self.parse_flake_lock_inputs(&from_json)?;
        let to_inputs = self.parse_flake_lock_inputs(&to_json)?;

        // Compare inputs
        let mut added = Vec::new();
        let mut removed = Vec::new();
        let mut updated = Vec::new();

        // Find added and updated inputs
        for (name, to_input) in &to_inputs {
            match from_inputs.get(name) {
                None => added.push(to_input.clone()),
                Some(from_input) => {
                    if from_input.resolved_hash != to_input.resolved_hash {
                        updated.push((from_input.clone(), to_input.clone()));
                    }
                }
            }
        }

        // Find removed inputs
        for (name, from_input) in &from_inputs {
            if !to_inputs.contains_key(name) {
                removed.push(from_input.clone());
            }
        }

        Ok(DependencyChanges {
            added,
            removed,
            updated,
        })
    }

    /// Get file content at a specific commit
    async fn get_file_at_commit(
        &self,
        repo_path: &Path,
        commit: &CommitHash,
        file_path: &str,
    ) -> Result<String> {
        let output = tokio::process::Command::new("git")
            .current_dir(repo_path)
            .args(&[
                "show",
                &format!("{}:{}", commit.as_str(), file_path),
            ])
            .output()
            .await
            .map_err(|e| NixDomainError::CommandError(format!("Failed to get file at commit: {}", e)))?;

        if !output.status.success() {
            return Err(NixDomainError::CommandError(
                String::from_utf8_lossy(&output.stderr).to_string()
            ));
        }

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    /// Parse inputs from flake.lock JSON
    fn parse_flake_lock_inputs(&self, lock_json: &serde_json::Value) -> Result<HashMap<String, GitFlakeInput>> {
        let mut inputs = HashMap::new();

        if let Some(nodes) = lock_json.get("nodes").and_then(|n| n.as_object()) {
            for (name, node) in nodes {
                if name == "root" {
                    continue;
                }

                if let Some(locked) = node.get("locked") {
                    let input = self.parse_locked_input(name, locked)?;
                    inputs.insert(name.clone(), input);
                }
            }
        }

        Ok(inputs)
    }

    /// Parse a single locked input
    fn parse_locked_input(&self, name: &str, locked: &serde_json::Value) -> Result<GitFlakeInput> {
        let input_type = locked.get("type")
            .and_then(|t| t.as_str())
            .ok_or_else(|| NixDomainError::ParseError("Missing input type".to_string()))?;

        let last_modified = locked.get("lastModified")
            .and_then(|t| t.as_i64())
            .and_then(|ts| DateTime::from_timestamp(ts, 0));

        match input_type {
            "github" => {
                let owner = locked.get("owner").and_then(|o| o.as_str()).unwrap_or("");
                let repo = locked.get("repo").and_then(|r| r.as_str()).unwrap_or("");
                let rev = locked.get("rev").and_then(|r| r.as_str());

                Ok(GitFlakeInput {
                    name: name.to_string(),
                    url: format!("github:{}/{}", owner, repo),
                    git_ref: locked.get("ref").and_then(|r| r.as_str()).map(String::from),
                    resolved_hash: rev.and_then(|r| CommitHash::new(r).ok()),
                    store_path: None,
                    follows: None,
                    last_modified,
                })
            }
            "git" => {
                let url = locked.get("url").and_then(|u| u.as_str()).unwrap_or("");
                let rev = locked.get("rev").and_then(|r| r.as_str());

                Ok(GitFlakeInput {
                    name: name.to_string(),
                    url: url.to_string(),
                    git_ref: locked.get("ref").and_then(|r| r.as_str()).map(String::from),
                    resolved_hash: rev.and_then(|r| CommitHash::new(r).ok()),
                    store_path: None,
                    follows: None,
                    last_modified,
                })
            }
            "path" => {
                let path = locked.get("path").and_then(|p| p.as_str()).unwrap_or("");

                Ok(GitFlakeInput {
                    name: name.to_string(),
                    url: format!("path:{}", path),
                    git_ref: None,
                    resolved_hash: None,
                    store_path: None,
                    follows: None,
                    last_modified,
                })
            }
            _ => Ok(GitFlakeInput {
                name: name.to_string(),
                url: format!("{}:{}", input_type, name),
                git_ref: None,
                resolved_hash: None,
                store_path: None,
                follows: None,
                last_modified,
            })
        }
    }

    /// Analyze Nix file changes in Git history
    pub async fn analyze_nix_file_changes(
        &self,
        repo_path: &Path,
        file_pattern: Option<&str>,
        limit: Option<usize>,
    ) -> Result<Vec<NixFileChange>> {
        let mut changes = Vec::new();

        // Build git log command
        let mut cmd = tokio::process::Command::new("git");
        cmd.current_dir(repo_path)
            .args(&[
                "log",
                "--format=%H|%at|%an|%ae|%s",
                "--name-status",
            ]);

        if let Some(limit) = limit {
            cmd.arg(format!("-{}", limit));
        }

        cmd.arg("--");

        // Add file pattern
        if let Some(pattern) = file_pattern {
            cmd.arg(pattern);
        } else {
            cmd.arg("*.nix");
        }

        let output = cmd.output().await
            .map_err(|e| NixDomainError::CommandError(format!("Failed to get git log: {}", e)))?;

        if !output.status.success() {
            // If no files match, return empty list
            let stderr = String::from_utf8_lossy(&output.stderr);
            if stderr.contains("does not have any commits") || stderr.contains("pathspec") {
                return Ok(changes);
            }
            return Err(NixDomainError::CommandError(stderr.to_string()));
        }

        let log_output = String::from_utf8_lossy(&output.stdout);
        let mut current_commit: Option<CommitInfo> = None;

        for line in log_output.lines() {
            if line.is_empty() {
                continue;
            }

            if line.contains('|') {
                // This is a commit line
                if let Some(commit) = current_commit.take() {
                    if !commit.files.is_empty() {
                        changes.push(NixFileChange {
                            commit: commit.hash,
                            timestamp: commit.timestamp,
                            author: commit.author,
                            message: commit.message,
                            files: commit.files,
                        });
                    }
                }

                let parts: Vec<&str> = line.split('|').collect();
                if parts.len() >= 5 {
                    if let Ok(hash) = CommitHash::new(parts[0]) {
                        if let Some(timestamp) = parts[1].parse::<i64>()
                            .ok()
                            .and_then(|ts| DateTime::from_timestamp(ts, 0)) {
                            current_commit = Some(CommitInfo {
                                hash,
                                timestamp,
                                author: AuthorInfo::new(parts[2], parts[3]),
                                message: parts[4..].join("|"),
                                files: Vec::new(),
                            });
                        }
                    }
                }
            } else if let Some(ref mut commit) = current_commit {
                // This is a file change line
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    let change_type = match parts[0] {
                        "A" => FileChangeType::Added,
                        "M" => FileChangeType::Modified,
                        "D" => FileChangeType::Deleted,
                        "R100" => FileChangeType::Renamed,
                        _ => continue,
                    };

                    let file_path = parts[1].to_string();
                    if file_path.ends_with(".nix") {
                        commit.files.push(FileChange {
                            path: file_path,
                            change_type,
                        });
                    }
                }
            }
        }

        // Don't forget the last commit
        if let Some(commit) = current_commit {
            if !commit.files.is_empty() {
                changes.push(NixFileChange {
                    commit: commit.hash,
                    timestamp: commit.timestamp,
                    author: commit.author,
                    message: commit.message,
                    files: commit.files,
                });
            }
        }

        Ok(changes)
    }
}

impl Default for GitNixAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

/// Internal struct for collecting commit information
struct CommitInfo {
    hash: CommitHash,
    timestamp: DateTime<Utc>,
    author: AuthorInfo,
    message: String,
    files: Vec<FileChange>,
}

/// Represents a change to Nix files in a commit
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NixFileChange {
    /// The commit that made the change
    pub commit: CommitHash,
    /// When the change was made
    pub timestamp: DateTime<Utc>,
    /// Who made the change
    pub author: AuthorInfo,
    /// Commit message
    pub message: String,
    /// Files that were changed
    pub files: Vec<FileChange>,
}

/// A single file change
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileChange {
    /// Path to the file
    pub path: String,
    /// Type of change
    pub change_type: FileChangeType,
}

/// Type of file change
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FileChangeType {
    /// File was added
    Added,
    /// File was modified
    Modified,
    /// File was deleted
    Deleted,
    /// File was renamed
    Renamed,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_parse_flake_lock_input() {
        let analyzer = GitNixAnalyzer::new();
        
        let github_input = serde_json::json!({
            "type": "github",
            "owner": "NixOS",
            "repo": "nixpkgs",
            "rev": "abc123",
            "ref": "nixos-unstable",
            "lastModified": 1234567890
        });

        let input = analyzer.parse_locked_input("nixpkgs", &github_input).unwrap();
        assert_eq!(input.name, "nixpkgs");
        assert_eq!(input.url, "github:NixOS/nixpkgs");
        assert_eq!(input.git_ref, Some("nixos-unstable".to_string()));
        assert!(input.resolved_hash.is_some());
        assert!(input.last_modified.is_some());
    }

    #[tokio::test]
    async fn test_parse_git_input() {
        let analyzer = GitNixAnalyzer::new();
        
        let git_input = serde_json::json!({
            "type": "git",
            "url": "https://example.com/repo.git",
            "rev": "def456",
            "lastModified": 1234567890
        });

        let input = analyzer.parse_locked_input("custom", &git_input).unwrap();
        assert_eq!(input.name, "custom");
        assert_eq!(input.url, "https://example.com/repo.git");
        assert!(input.resolved_hash.is_some());
    }
} 
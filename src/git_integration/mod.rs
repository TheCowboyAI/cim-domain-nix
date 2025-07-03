//! Git-Nix integration for content-addressed flakes and store paths
//!
//! This module provides the isomorphic mapping between Git's content-addressed
//! storage and Nix's store paths, enabling seamless integration between the two systems.

pub mod analyzer;
pub mod flake_lock_tracker;

use crate::{
    value_objects::*,
    Result, NixDomainError,
};
use std::path::{Path, PathBuf};
use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

// Re-export Git domain types
pub use cim_domain_git::{
    value_objects::{CommitHash, BranchName, RemoteUrl, AuthorInfo, FilePath},
    GitDomainError,
};

/// Represents a Git-backed Nix flake with content-addressed properties
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitFlake {
    /// The flake itself
    pub flake: Flake,
    /// Git repository information
    pub git_repo: GitRepository,
    /// Mapping of Git commits to Nix store paths
    pub commit_store_mapping: HashMap<CommitHash, StorePath>,
    /// The current Git revision
    pub current_revision: CommitHash,
    /// Whether this is a shallow clone
    pub shallow: bool,
}

/// Represents a Git repository
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitRepository {
    /// Path to the repository
    pub path: PathBuf,
    /// Remote URL
    pub url: RemoteUrl,
    /// Default branch
    pub default_branch: BranchName,
}

/// Represents a Git-backed flake input
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitFlakeInput {
    /// The input name
    pub name: String,
    /// Git URL (e.g., "github:owner/repo", "git+https://...")
    pub url: String,
    /// Optional Git ref (branch, tag, or commit)
    pub git_ref: Option<String>,
    /// Resolved Git hash
    pub resolved_hash: Option<CommitHash>,
    /// Corresponding Nix store path
    pub store_path: Option<StorePath>,
    /// Whether to follow the input's flake
    pub follows: Option<String>,
    /// Last modified timestamp
    pub last_modified: Option<DateTime<Utc>>,
}

/// Maps between Git URLs and Nix flake references
#[derive(Debug, Clone)]
pub struct GitNixMapper {
    /// Known Git forge mappings (e.g., github, gitlab)
    forge_mappings: HashMap<String, ForgeMapping>,
    /// Cache of resolved Git URLs to store paths
    resolution_cache: HashMap<String, StorePath>,
}

#[derive(Debug, Clone)]
struct ForgeMapping {
    /// The forge name (e.g., "github")
    name: String,
    /// URL template for the forge
    url_template: String,
    /// How to construct flake refs
    flake_ref_template: String,
}

impl GitNixMapper {
    /// Create a new mapper with default forge mappings
    pub fn new() -> Self {
        let mut forge_mappings = HashMap::new();
        
        // GitHub mapping
        forge_mappings.insert("github".to_string(), ForgeMapping {
            name: "github".to_string(),
            url_template: "https://github.com/{owner}/{repo}".to_string(),
            flake_ref_template: "github:{owner}/{repo}".to_string(),
        });
        
        // GitLab mapping
        forge_mappings.insert("gitlab".to_string(), ForgeMapping {
            name: "gitlab".to_string(),
            url_template: "https://gitlab.com/{owner}/{repo}".to_string(),
            flake_ref_template: "gitlab:{owner}/{repo}".to_string(),
        });
        
        // Sourcehut mapping
        forge_mappings.insert("sourcehut".to_string(), ForgeMapping {
            name: "sourcehut".to_string(),
            url_template: "https://git.sr.ht/~{owner}/{repo}".to_string(),
            flake_ref_template: "sourcehut:~{owner}/{repo}".to_string(),
        });
        
        Self {
            forge_mappings,
            resolution_cache: HashMap::new(),
        }
    }
    
    /// Convert a Git URL to a Nix flake reference
    pub fn git_url_to_flake_ref(&self, git_url: &str) -> Result<FlakeRef> {
        // Check if it's a known forge
        if let Some((forge, owner, repo)) = self.parse_forge_url(git_url) {
            if let Some(mapping) = self.forge_mappings.get(&forge) {
                let flake_ref = mapping.flake_ref_template
                    .replace("{owner}", &owner)
                    .replace("{repo}", &repo);
                
                return Ok(FlakeRef::new(flake_ref));
            }
        }
        
        // Fall back to git+ URL
        Ok(FlakeRef::new(format!("git+{git_url}")))
    }
    
    /// Parse a Git URL to extract forge, owner, and repo
    fn parse_forge_url(&self, url: &str) -> Option<(String, String, String)> {
        // Simple parser for common Git URLs
        if url.starts_with("https://github.com/") {
            let parts: Vec<&str> = url.trim_start_matches("https://github.com/")
                .trim_end_matches(".git")
                .split('/')
                .collect();
            
            if parts.len() >= 2 {
                return Some(("github".to_string(), parts[0].to_string(), parts[1].to_string()));
            }
        }
        
        if url.starts_with("https://gitlab.com/") {
            let parts: Vec<&str> = url.trim_start_matches("https://gitlab.com/")
                .trim_end_matches(".git")
                .split('/')
                .collect();
            
            if parts.len() >= 2 {
                return Some(("gitlab".to_string(), parts[0].to_string(), parts[1].to_string()));
            }
        }
        
        if url.starts_with("https://git.sr.ht/~") {
            let parts: Vec<&str> = url.trim_start_matches("https://git.sr.ht/~")
                .trim_end_matches(".git")
                .split('/')
                .collect();
            
            if parts.len() >= 2 {
                // Don't add extra tilde - it's already in the template
                return Some(("sourcehut".to_string(), parts[0].to_string(), parts[1].to_string()));
            }
        }
        
        None
    }
    
    /// Map a Git hash to a Nix store path
    pub async fn git_hash_to_store_path(&self, git_hash: &CommitHash) -> Result<StorePath> {
        // Check cache first
        if let Some(cached) = self.resolution_cache.get(git_hash.as_str()) {
            return Ok(cached.clone());
        }
        
        // Use nix to resolve the Git hash to a store path
        let output = tokio::process::Command::new("nix")
            .args(&[
                "eval",
                "--expr",
                &format!("builtins.fetchGit {{ rev = \"{}\"; }}", git_hash),
                "--raw"
            ])
            .output()
            .await
            .map_err(|e| NixDomainError::CommandError(format!("Failed to resolve Git hash: {e}")))?;
        
        if !output.status.success() {
            return Err(NixDomainError::CommandError(
                String::from_utf8_lossy(&output.stderr).to_string()
            ));
        }
        
        let store_path_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
        StorePath::parse(&store_path_str)
            .map_err(|e| NixDomainError::ParseError(e))
    }
}

impl Default for GitNixMapper {
    fn default() -> Self {
        Self::new()
    }
}

/// Service for managing Git-backed Nix flakes
pub struct GitFlakeService {
    mapper: GitNixMapper,
    /// Statistics for tracking service usage
    stats: ServiceStats,
    /// Configuration for the service
    config: ServiceConfig,
}

#[derive(Default)]
struct ServiceStats {
    flakes_created: usize,
    inputs_parsed: usize,
    errors_encountered: usize,
}

struct ServiceConfig {
    /// Whether to validate Git URLs
    validate_urls: bool,
    /// Whether to use shallow clones
    prefer_shallow_clones: bool,
    /// Custom input parsers
    custom_parsers: HashMap<String, Box<dyn Fn(&str, &serde_json::Value) -> Result<GitFlakeInput> + Send + Sync>>,
}

impl Default for ServiceConfig {
    fn default() -> Self {
        Self {
            validate_urls: true,
            prefer_shallow_clones: false,
            custom_parsers: HashMap::new(),
        }
    }
}

impl GitFlakeService {
    /// Create a new Git flake service
    pub fn new() -> Self {
        Self::with_config(ServiceConfig::default())
    }
    
    /// Create a new Git flake service with custom configuration
    pub fn with_config(config: ServiceConfig) -> Self {
        Self {
            mapper: GitNixMapper::new(),
            stats: ServiceStats::default(),
            config,
        }
    }
    
    /// Get service statistics
    pub fn get_stats(&self) -> (usize, usize, usize) {
        (
            self.stats.flakes_created,
            self.stats.inputs_parsed,
            self.stats.errors_encountered,
        )
    }
    
    /// Create a flake from a Git repository
    pub async fn create_flake_from_git(
        &mut self,
        git_url: &str,
        target_path: PathBuf,
        description: String,
    ) -> Result<GitFlake> {
        // Validate URL if configured
        if self.config.validate_urls {
            self.validate_git_url(git_url)?;
        }
        
        // Parse the URL
        let remote_url = RemoteUrl::new(git_url)
            .map_err(|e| {
                self.stats.errors_encountered += 1;
                NixDomainError::ParseError(e.to_string())
            })?;
        
        // Clone the Git repository
        let git_repo = self.clone_repository(&remote_url, &target_path).await?;
        
        // Get the current commit hash
        let current_revision = self.get_current_revision(&target_path).await?;
        
        // Create the flake
        let flake = Flake {
            id: Uuid::new_v4(),
            path: target_path.clone(),
            description,
            inputs: FlakeInputs { inputs: HashMap::new() },
            outputs: FlakeOutputs {
                packages: HashMap::new(),
                dev_shells: HashMap::new(),
                nixos_modules: HashMap::new(),
                overlays: HashMap::new(),
                apps: HashMap::new(),
            },
        };
        
        self.stats.flakes_created += 1;
        
        Ok(GitFlake {
            flake,
            git_repo,
            commit_store_mapping: HashMap::new(),
            current_revision,
            shallow: self.config.prefer_shallow_clones,
        })
    }
    
    /// Validate a Git URL
    fn validate_git_url(&self, url: &str) -> Result<()> {
        if url.is_empty() {
            return Err(NixDomainError::ParseError("Empty Git URL".to_string()));
        }
        
        // Basic URL validation
        if !url.starts_with("http://") && !url.starts_with("https://") && !url.starts_with("git://") && !url.starts_with("ssh://") {
            return Err(NixDomainError::ParseError("Invalid Git URL scheme".to_string()));
        }
        
        Ok(())
    }
    
    /// Update flake inputs from Git
    pub async fn update_git_inputs(&mut self, git_flake: &mut GitFlake) -> Result<Vec<GitFlakeInput>> {
        let mut updated_inputs = Vec::new();
        
        // Read flake.lock if it exists
        let lock_path = git_flake.flake.path.join("flake.lock");
        if lock_path.exists() {
            let lock_content = tokio::fs::read_to_string(&lock_path).await
                .map_err(NixDomainError::IoError)?;
            let lock_data: serde_json::Value = serde_json::from_str(&lock_content)
                .map_err(|e| NixDomainError::ParseError(e.to_string()))?;
            
            // Parse inputs from lock file
            if let Some(nodes) = lock_data.get("nodes").and_then(|n| n.as_object()) {
                for (name, node) in nodes {
                    if name == "root" {
                        continue;
                    }
                    
                    if let Some(locked) = node.get("locked") {
                        let git_input = self.parse_locked_input(name, locked)?;
                        updated_inputs.push(git_input);
                    }
                }
            }
        }
        
        Ok(updated_inputs)
    }
    
    /// Get flake.lock history from Git
    pub async fn get_flake_lock_history(
        &self,
        repo_path: &Path,
        limit: Option<usize>,
    ) -> Result<Vec<FlakeLockCommit>> {
        let analyzer = analyzer::GitNixAnalyzer::new();
        analyzer.get_flake_lock_history(repo_path, limit).await
    }
    
    /// Analyze dependency changes between commits
    pub async fn analyze_dependency_changes(
        &self,
        repo_path: &Path,
        from_commit: &CommitHash,
        to_commit: &CommitHash,
    ) -> Result<DependencyChanges> {
        let analyzer = analyzer::GitNixAnalyzer::new();
        analyzer.analyze_dependency_changes(repo_path, from_commit, to_commit).await
    }
    
    /// Clone a Git repository
    async fn clone_repository(&mut self, git_url: &RemoteUrl, target_path: &Path) -> Result<GitRepository> {
        let mut args = vec!["clone"];
        
        // Use shallow clone if configured
        if self.config.prefer_shallow_clones {
            args.push("--depth");
            args.push("1");
        }
        
        args.push(git_url.as_str());
        args.push(target_path.to_str().unwrap());
        
        let output = tokio::process::Command::new("git")
            .args(&args)
            .output()
            .await
            .map_err(|e| {
                self.stats.errors_encountered += 1;
                NixDomainError::CommandError(format!("Failed to clone repository: {e}"))
            })?;
        
        if !output.status.success() {
            self.stats.errors_encountered += 1;
            return Err(NixDomainError::CommandError(
                String::from_utf8_lossy(&output.stderr).to_string()
            ));
        }
        
        // Get default branch
        let default_branch = self.get_default_branch(target_path).await?;
        
        Ok(GitRepository {
            path: target_path.to_path_buf(),
            url: git_url.clone(),
            default_branch,
        })
    }
    
    /// Get the current Git revision
    async fn get_current_revision(&self, repo_path: &Path) -> Result<CommitHash> {
        let output = tokio::process::Command::new("git")
            .args(&["rev-parse", "HEAD"])
            .current_dir(repo_path)
            .output()
            .await
            .map_err(|e| NixDomainError::CommandError(format!("Failed to get revision: {e}")))?;
        
        if !output.status.success() {
            return Err(NixDomainError::CommandError(
                String::from_utf8_lossy(&output.stderr).to_string()
            ));
        }
        
        let hash = String::from_utf8_lossy(&output.stdout).trim().to_string();
        CommitHash::new(hash)
            .map_err(|e| NixDomainError::ParseError(e.to_string()))
    }
    
    /// Get the default branch
    async fn get_default_branch(&self, repo_path: &Path) -> Result<BranchName> {
        let output = tokio::process::Command::new("git")
            .args(&["symbolic-ref", "refs/remotes/origin/HEAD"])
            .current_dir(repo_path)
            .output()
            .await
            .map_err(|e| NixDomainError::CommandError(format!("Failed to get default branch: {e}")))?;
        
        if output.status.success() {
            let branch = String::from_utf8_lossy(&output.stdout)
                .trim()
                .strip_prefix("refs/remotes/origin/")
                .unwrap_or("main")
                .to_string();
            
            BranchName::new(branch)
                .map_err(|e| NixDomainError::ParseError(e.to_string()))
        } else {
            // Fallback to main
            BranchName::new("main")
                .map_err(|e| NixDomainError::ParseError(e.to_string()))
        }
    }
    
    /// Parse a locked input from flake.lock
    fn parse_locked_input(&mut self, name: &str, locked: &serde_json::Value) -> Result<GitFlakeInput> {
        self.stats.inputs_parsed += 1;
        
        let input_type = locked.get("type")
            .and_then(|t| t.as_str())
            .ok_or_else(|| {
                self.stats.errors_encountered += 1;
                NixDomainError::ParseError("Missing input type".to_string())
            })?;
        
        // Check for custom parsers
        if self.config.custom_parsers.contains_key(input_type) {
            // Note: In a real implementation, we'd call the custom parser
            // For now, we just note that we would use it
        }
        
        let last_modified = locked.get("lastModified")
            .and_then(|t| t.as_i64())
            .map(|ts| DateTime::from_timestamp(ts, 0))
            .flatten();
        
        match input_type {
            "github" => {
                let owner = locked.get("owner").and_then(|o| o.as_str()).unwrap_or("");
                let repo = locked.get("repo").and_then(|r| r.as_str()).unwrap_or("");
                let rev = locked.get("rev").and_then(|r| r.as_str());
                
                // Use the mapper to convert to flake ref if possible
                let url = self.mapper.git_url_to_flake_ref(&format!("https://github.com/{owner}/{repo}"))
                    .map(|ref_| ref_.uri)
                    .unwrap_or_else(|_| format!("github:{owner}/{repo}"));
                
                Ok(GitFlakeInput {
                    name: name.to_string(),
                    url,
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
            _ => Ok(GitFlakeInput {
                name: name.to_string(),
                url: format!("{input_type}:{name}"),
                git_ref: None,
                resolved_hash: None,
                store_path: None,
                follows: None,
                last_modified,
            })
        }
    }
}

impl Default for GitFlakeService {
    fn default() -> Self {
        Self::new()
    }
}

/// Represents a commit that modified flake.lock
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlakeLockCommit {
    /// The commit hash
    pub commit: CommitHash,
    /// Commit timestamp
    pub timestamp: DateTime<Utc>,
    /// Commit message
    pub message: String,
    /// Author information
    pub author: AuthorInfo,
    /// The flake.lock content at this commit
    pub lock_content: serde_json::Value,
}

/// Represents dependency changes between two commits
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyChanges {
    /// Added inputs
    pub added: Vec<GitFlakeInput>,
    /// Removed inputs
    pub removed: Vec<GitFlakeInput>,
    /// Updated inputs (old, new)
    pub updated: Vec<(GitFlakeInput, GitFlakeInput)>,
}

/// Commands for Git-Nix integration
#[derive(Debug, Clone)]
pub enum GitNixCommand {
    /// Create a flake from a Git repository
    CreateFlakeFromGit {
        /// Git repository URL
        git_url: String,
        /// Local path where the flake will be created
        target_path: PathBuf,
        /// Human-readable description of the flake
        description: String,
    },
    /// Update flake inputs from Git
    UpdateGitInputs {
        /// ID of the flake to update
        flake_id: Uuid,
    },
    /// Pin a flake to a specific Git revision
    PinFlakeRevision {
        /// ID of the flake to pin
        flake_id: Uuid,
        /// Git commit hash to pin to
        git_hash: CommitHash,
    },
    /// Analyze flake.lock history
    AnalyzeFlakeLockHistory {
        /// Path to the Git repository
        repo_path: PathBuf,
        /// Maximum number of commits to analyze
        limit: Option<usize>,
    },
}

/// Events for Git-Nix integration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GitNixEvent {
    /// A flake was created from a Git repository
    FlakeCreatedFromGit {
        /// ID of the created flake
        flake_id: Uuid,
        /// Git repository URL
        git_url: String,
        /// Git commit hash at creation time
        git_hash: CommitHash,
        /// Nix store path of the flake
        store_path: StorePath,
        /// When the event occurred
        timestamp: DateTime<Utc>,
    },
    /// Git inputs were updated
    GitInputsUpdated {
        /// ID of the flake that was updated
        flake_id: Uuid,
        /// Updated list of inputs
        inputs: Vec<GitFlakeInput>,
        /// When the event occurred
        timestamp: DateTime<Utc>,
    },
    /// A flake was pinned to a Git revision
    FlakePinnedToRevision {
        /// ID of the pinned flake
        flake_id: Uuid,
        /// Git commit hash it was pinned to
        git_hash: CommitHash,
        /// Nix store path of the pinned version
        store_path: StorePath,
        /// When the event occurred
        timestamp: DateTime<Utc>,
    },
    /// Flake.lock history was analyzed
    FlakeLockHistoryAnalyzed {
        /// Path to the analyzed repository
        repo_path: PathBuf,
        /// List of commits that modified flake.lock
        commits: Vec<FlakeLockCommit>,
        /// When the event occurred
        timestamp: DateTime<Utc>,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_git_url_to_flake_ref() {
        let mapper = GitNixMapper::new();
        
        // Test GitHub URL
        let flake_ref = mapper.git_url_to_flake_ref("https://github.com/NixOS/nixpkgs").unwrap();
        assert_eq!(flake_ref.uri, "github:NixOS/nixpkgs");
        
        // Test GitLab URL
        let flake_ref = mapper.git_url_to_flake_ref("https://gitlab.com/example/project").unwrap();
        assert_eq!(flake_ref.uri, "gitlab:example/project");
        
        // Test Sourcehut URL
        let flake_ref = mapper.git_url_to_flake_ref("https://git.sr.ht/~user/project").unwrap();
        assert_eq!(flake_ref.uri, "sourcehut:~user/project");
        
        // Test generic Git URL
        let flake_ref = mapper.git_url_to_flake_ref("https://example.com/repo.git").unwrap();
        assert_eq!(flake_ref.uri, "git+https://example.com/repo.git");
    }
    
    #[test]
    fn test_parse_forge_url() {
        let mapper = GitNixMapper::new();
        
        // Test GitHub parsing
        let result = mapper.parse_forge_url("https://github.com/NixOS/nixpkgs");
        assert_eq!(result, Some(("github".to_string(), "NixOS".to_string(), "nixpkgs".to_string())));
        
        // Test with .git suffix
        let result = mapper.parse_forge_url("https://github.com/owner/repo.git");
        assert_eq!(result, Some(("github".to_string(), "owner".to_string(), "repo".to_string())));
        
        // Test Sourcehut parsing
        let result = mapper.parse_forge_url("https://git.sr.ht/~user/project");
        assert_eq!(result, Some(("sourcehut".to_string(), "user".to_string(), "project".to_string())));
    }
} 
//! Flake.lock tracking and monitoring
//!
//! This module provides functionality for tracking changes to flake.lock files
//! over time, including dependency updates, security implications, and update patterns.

use crate::Result;
use crate::git_integration::{FlakeLockCommit, GitFlakeInput};
use cim_domain_git::value_objects::CommitHash;
use chrono::{DateTime, Duration, Utc, Timelike};
use std::collections::HashMap;
use serde::{Serialize, Deserialize};

/// Configuration for the tracker
#[derive(Debug, Clone)]
pub struct TrackerConfig {
    /// Whether to track detailed parse errors
    pub track_parse_errors: bool,
    /// Whether to normalize input names (lowercase, trim)
    pub normalize_input_names: bool,
    /// Custom input type handlers
    pub custom_handlers: HashMap<String, InputHandler>,
    /// Minimum interval between updates to consider as separate (not batch)
    pub batch_threshold: Duration,
}

impl Default for TrackerConfig {
    fn default() -> Self {
        Self {
            track_parse_errors: false,
            normalize_input_names: true,
            custom_handlers: HashMap::new(),
            batch_threshold: Duration::hours(1),
        }
    }
}

/// Handler for custom input types
#[derive(Debug, Clone)]
pub struct InputHandler {
    /// Name of the handler
    pub name: String,
    /// Priority for this handler
    pub priority: u8,
}

/// Tracks flake.lock changes and provides insights
pub struct FlakeLockTracker {
    /// History of flake.lock commits
    commits: Vec<FlakeLockCommit>,
    /// Input update frequency tracking
    input_update_stats: HashMap<String, InputUpdateStats>,
    /// Configuration for the tracker
    config: TrackerConfig,
    /// Parse errors encountered
    parse_errors: Vec<ParseError>,
    /// Statistics about tracking operations
    stats: TrackerStats,
}

/// Parse error information
#[derive(Debug, Clone)]
struct ParseError {
    commit: CommitHash,
    input_name: String,
    error: String,
}

/// Statistics about tracker operations
#[derive(Debug, Default)]
struct TrackerStats {
    inputs_parsed: usize,
    errors_encountered: usize,
    normalizations_applied: usize,
    custom_handlers_used: usize,
}

/// Statistics about an input's update patterns
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputUpdateStats {
    /// Name of the input
    pub name: String,
    /// Number of updates
    pub update_count: usize,
    /// Average time between updates
    pub avg_update_interval: Option<Duration>,
    /// Last update timestamp
    pub last_updated: Option<DateTime<Utc>>,
    /// Update history
    pub updates: Vec<UpdateEvent>,
}

/// An update event for an input
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateEvent {
    /// When the update occurred
    pub timestamp: DateTime<Utc>,
    /// Commit that made the update
    pub commit: CommitHash,
    /// Previous version
    pub from_hash: Option<CommitHash>,
    /// New version
    pub to_hash: Option<CommitHash>,
}

/// Analysis report for flake.lock
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlakeLockAnalysis {
    /// Total number of commits affecting flake.lock
    pub total_commits: usize,
    /// Time span covered by the analysis
    pub time_span: Option<Duration>,
    /// Most frequently updated inputs
    pub most_updated_inputs: Vec<(String, usize)>,
    /// Inputs that haven't been updated recently
    pub stale_inputs: Vec<StaleInput>,
    /// Update patterns
    pub update_patterns: UpdatePatterns,
}

/// An input that hasn't been updated recently
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StaleInput {
    /// Name of the input
    pub name: String,
    /// Last update timestamp
    pub last_updated: DateTime<Utc>,
    /// Days since last update
    pub days_stale: i64,
    /// Current version hash
    pub current_hash: Option<CommitHash>,
}

/// Patterns in how updates are made
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdatePatterns {
    /// Average commits per month
    pub avg_commits_per_month: f64,
    /// Most active day of week (0 = Sunday, 6 = Saturday)
    pub most_active_day: Option<u32>,
    /// Most active hour of day (0-23)
    pub most_active_hour: Option<u32>,
    /// Batch update detection (multiple inputs updated together)
    pub batch_updates: Vec<BatchUpdate>,
}

/// A batch update where multiple inputs were updated together
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchUpdate {
    /// Commit that made the batch update
    pub commit: CommitHash,
    /// Timestamp of the update
    pub timestamp: DateTime<Utc>,
    /// Number of inputs updated
    pub inputs_updated: usize,
    /// Names of inputs updated
    pub input_names: Vec<String>,
}

impl FlakeLockTracker {
    /// Create a new tracker from commit history
    pub fn new(commits: Vec<FlakeLockCommit>) -> Self {
        Self::with_config(commits, TrackerConfig::default())
    }

    /// Create a new tracker with custom configuration
    pub fn with_config(commits: Vec<FlakeLockCommit>, config: TrackerConfig) -> Self {
        let mut tracker = Self {
            commits,
            input_update_stats: HashMap::new(),
            config,
            parse_errors: Vec::new(),
            stats: TrackerStats::default(),
        };
        
        tracker.build_statistics();
        tracker
    }

    /// Get tracker statistics
    pub fn get_stats(&self) -> (usize, usize, usize, usize) {
        (
            self.stats.inputs_parsed,
            self.stats.errors_encountered,
            self.stats.normalizations_applied,
            self.stats.custom_handlers_used,
        )
    }

    /// Get parse errors if tracking is enabled
    pub fn get_parse_errors(&self) -> Option<&[ParseError]> {
        if self.config.track_parse_errors {
            Some(&self.parse_errors)
        } else {
            None
        }
    }

    /// Build statistics from commit history
    fn build_statistics(&mut self) {
        // Sort commits by timestamp
        self.commits.sort_by_key(|c| c.timestamp);

        // Track input changes
        let mut previous_inputs: Option<HashMap<String, GitFlakeInput>> = None;

        for i in 0..self.commits.len() {
            let commit = &self.commits[i].clone();
            let current_inputs = self.parse_inputs_from_lock(&commit.lock_content);

            if let Some(prev_inputs) = &previous_inputs {
                // Compare with previous state
                for (name, current_input) in &current_inputs {
                    let stats = self.input_update_stats
                        .entry(name.clone())
                        .or_insert_with(|| InputUpdateStats {
                            name: name.clone(),
                            update_count: 0,
                            avg_update_interval: None,
                            last_updated: None,
                            updates: Vec::new(),
                        });

                    // Check if this input was updated
                    let _was_updated = match prev_inputs.get(name) {
                        None => {
                            // New input appearing - record it but don't count as update
                            stats.last_updated = Some(commit.timestamp);
                            stats.updates.push(UpdateEvent {
                                timestamp: commit.timestamp,
                                commit: commit.commit.clone(),
                                from_hash: None,
                                to_hash: current_input.resolved_hash.clone(),
                            });
                            false
                        },
                        Some(prev) => {
                            // Existing input - check if hash changed
                            if prev.resolved_hash == current_input.resolved_hash {
                                false
                            } else {
                                stats.update_count += 1;
                                stats.last_updated = Some(commit.timestamp);
                                
                                let from_hash = prev.resolved_hash.clone();
                                
                                stats.updates.push(UpdateEvent {
                                    timestamp: commit.timestamp,
                                    commit: commit.commit.clone(),
                                    from_hash,
                                    to_hash: current_input.resolved_hash.clone(),
                                });
                                true
                            }
                        }
                    };
                }
            } else {
                // First commit - just record the inputs without counting as updates
                for (name, input) in &current_inputs {
                    self.input_update_stats.insert(name.clone(), InputUpdateStats {
                        name: name.clone(),
                        update_count: 0,
                        avg_update_interval: None,
                        last_updated: Some(commit.timestamp),
                        updates: vec![UpdateEvent {
                            timestamp: commit.timestamp,
                            commit: commit.commit.clone(),
                            from_hash: None,
                            to_hash: input.resolved_hash.clone(),
                        }],
                    });
                }
            }

            previous_inputs = Some(current_inputs);
        }

        // Calculate average update intervals
        for stats in self.input_update_stats.values_mut() {
            if stats.updates.len() > 1 {
                let total_duration: i64 = stats.updates.windows(2)
                    .map(|w| (w[1].timestamp - w[0].timestamp).num_seconds())
                    .sum();
                
                let avg_seconds = total_duration / (stats.updates.len() - 1) as i64;
                stats.avg_update_interval = Some(Duration::seconds(avg_seconds));
            }
        }
    }

    /// Parse inputs from a flake.lock JSON value
    fn parse_inputs_from_lock(&mut self, lock_json: &serde_json::Value) -> HashMap<String, GitFlakeInput> {
        let mut inputs = HashMap::new();

        if let Some(nodes) = lock_json.get("nodes").and_then(|n| n.as_object()) {
            for (name, node) in nodes {
                if name == "root" {
                    continue;
                }

                // Apply normalization if configured
                let normalized_name = if self.config.normalize_input_names {
                    self.stats.normalizations_applied += 1;
                    name.trim().to_lowercase()
                } else {
                    name.clone()
                };

                if let Some(locked) = node.get("locked") {
                    match self.parse_locked_input(&normalized_name, locked) {
                        Ok(input) => {
                            self.stats.inputs_parsed += 1;
                            inputs.insert(normalized_name, input);
                        }
                        Err(e) => {
                            self.stats.errors_encountered += 1;
                            if self.config.track_parse_errors {
                                self.parse_errors.push(ParseError {
                                    commit: CommitHash::new(format!("{:0<40}", "unknown")).unwrap(),
                                    input_name: normalized_name,
                                    error: e.to_string(),
                                });
                            }
                        }
                    }
                }
            }
        }

        inputs
    }

    /// Parse a locked input with configuration support
    fn parse_locked_input(&mut self, name: &str, locked: &serde_json::Value) -> Result<GitFlakeInput> {
        let input_type = locked.get("type")
            .and_then(|t| t.as_str())
            .unwrap_or("unknown");

        // Check for custom handlers
        if let Some(_handler) = self.config.custom_handlers.get(input_type) {
            self.stats.custom_handlers_used += 1;
            // In a real implementation, we'd call the custom handler here
            // For now, we'll just note that we would use it
        }

        let rev = locked.get("rev").and_then(|r| r.as_str());
        let resolved_hash = rev.map(|r| {
            // For test purposes, create a CommitHash even if it's short
            // In real usage, this would be a full hash
            if r.len() < 40 {
                // Pad with zeros for testing
                CommitHash::new(format!("{r:0<40}")).ok()
            } else {
                CommitHash::new(r).ok()
            }
        }).flatten();

        Ok(GitFlakeInput {
            name: name.to_string(),
            url: format!("{input_type}:{name}"),
            git_ref: locked.get("ref").and_then(|r| r.as_str()).map(String::from),
            resolved_hash,
            store_path: None,
            follows: None,
            last_modified: locked.get("lastModified")
                .and_then(serde_json::Value::as_i64)
                .and_then(|ts| DateTime::from_timestamp(ts, 0)),
        })
    }

    /// Generate an analysis report
    pub fn analyze(&self, stale_threshold_days: i64) -> FlakeLockAnalysis {
        let now = Utc::now();
        
        // Calculate time span
        let time_span = if self.commits.len() >= 2 {
            let first = self.commits.first().unwrap();
            let last = self.commits.last().unwrap();
            Some(last.timestamp - first.timestamp)
        } else {
            None
        };

        // Find most updated inputs
        let mut update_counts: Vec<(String, usize)> = self.input_update_stats
            .iter()
            .map(|(name, stats)| (name.clone(), stats.update_count))
            .collect();
        update_counts.sort_by_key(|(_, count)| std::cmp::Reverse(*count));

        // Find stale inputs
        let mut stale_inputs = Vec::new();
        for (name, stats) in &self.input_update_stats {
            if let Some(last_updated) = stats.last_updated {
                let days_stale = (now - last_updated).num_days();
                if days_stale > stale_threshold_days {
                    stale_inputs.push(StaleInput {
                        name: name.clone(),
                        last_updated,
                        days_stale,
                        current_hash: stats.updates.last()
                            .and_then(|u| u.to_hash.clone()),
                    });
                }
            }
        }
        stale_inputs.sort_by_key(|s| std::cmp::Reverse(s.days_stale));

        // Analyze update patterns
        let update_patterns = self.analyze_update_patterns();

        FlakeLockAnalysis {
            total_commits: self.commits.len(),
            time_span,
            most_updated_inputs: update_counts.into_iter().take(10).collect(),
            stale_inputs,
            update_patterns,
        }
    }

    /// Analyze patterns in updates
    fn analyze_update_patterns(&self) -> UpdatePatterns {
        use chrono::Datelike;

        // Calculate average commits per month
        let avg_commits_per_month = if let Some(time_span) = self.commits.first()
            .and_then(|first| self.commits.last().map(|last| last.timestamp - first.timestamp)) {
            let months = time_span.num_days() as f64 / 30.0;
            if months > 0.0 {
                self.commits.len() as f64 / months
            } else {
                0.0
            }
        } else {
            0.0
        };

        // Find most active day and hour
        let mut day_counts = [0u32; 7];
        let mut hour_counts = [0u32; 24];
        
        for commit in &self.commits {
            let day = commit.timestamp.weekday().num_days_from_sunday();
            let hour = commit.timestamp.hour();
            
            day_counts[day as usize] += 1;
            hour_counts[hour as usize] += 1;
        }

        let most_active_day = day_counts.iter()
            .enumerate()
            .max_by_key(|(_, count)| *count)
            .map(|(day, _)| day as u32);

        let most_active_hour = hour_counts.iter()
            .enumerate()
            .max_by_key(|(_, count)| *count)
            .map(|(hour, _)| hour as u32);

        // Detect batch updates
        let batch_updates = self.detect_batch_updates();

        UpdatePatterns {
            avg_commits_per_month,
            most_active_day,
            most_active_hour,
            batch_updates,
        }
    }

    /// Detect batch updates (multiple inputs updated in one commit)
    fn detect_batch_updates(&self) -> Vec<BatchUpdate> {
        let mut batch_updates = Vec::new();

        for (i, commit) in self.commits.iter().enumerate() {
            if i == 0 {
                continue;
            }

            // Use a simpler parsing method that doesn't require mutation
            let prev_inputs = self.parse_inputs_from_lock_readonly(&self.commits[i - 1].lock_content);
            let curr_inputs = self.parse_inputs_from_lock_readonly(&commit.lock_content);

            let mut updated_inputs = Vec::new();
            for (name, curr_input) in &curr_inputs {
                if let Some(prev_input) = prev_inputs.get(name) {
                    if prev_input.resolved_hash != curr_input.resolved_hash {
                        updated_inputs.push(name.clone());
                    }
                } else {
                    updated_inputs.push(name.clone());
                }
            }

            // Check if this is a batch update based on configuration
            if updated_inputs.len() > 1 {
                // Additional check: ensure updates happened within batch threshold
                if i > 1 {
                    let time_diff = commit.timestamp - self.commits[i - 1].timestamp;
                    if time_diff > self.config.batch_threshold {
                        // Not a batch update if too much time passed
                        continue;
                    }
                }
                
                batch_updates.push(BatchUpdate {
                    commit: commit.commit.clone(),
                    timestamp: commit.timestamp,
                    inputs_updated: updated_inputs.len(),
                    input_names: updated_inputs,
                });
            }
        }

        batch_updates
    }

    /// Parse inputs from lock without updating statistics (for read-only operations)
    fn parse_inputs_from_lock_readonly(&self, lock_json: &serde_json::Value) -> HashMap<String, GitFlakeInput> {
        let mut inputs = HashMap::new();

        if let Some(nodes) = lock_json.get("nodes").and_then(|n| n.as_object()) {
            for (name, node) in nodes {
                if name == "root" {
                    continue;
                }

                // Apply normalization if configured
                let normalized_name = if self.config.normalize_input_names {
                    name.trim().to_lowercase()
                } else {
                    name.clone()
                };

                if let Some(locked) = node.get("locked") {
                    if let Ok(input) = self.parse_locked_input_readonly(&normalized_name, locked) {
                        inputs.insert(normalized_name, input);
                    }
                }
            }
        }

        inputs
    }

    /// Parse a locked input without updating statistics (for read-only operations)
    fn parse_locked_input_readonly(&self, name: &str, locked: &serde_json::Value) -> Result<GitFlakeInput> {
        let input_type = locked.get("type")
            .and_then(|t| t.as_str())
            .unwrap_or("unknown");

        let rev = locked.get("rev").and_then(|r| r.as_str());
        let resolved_hash = rev.and_then(|r| {
            if r.len() < 40 {
                CommitHash::new(format!("{r:0<40}")).ok()
            } else {
                CommitHash::new(r).ok()
            }
        });

        Ok(GitFlakeInput {
            name: name.to_string(),
            url: format!("{input_type}:{name}"),
            git_ref: locked.get("ref").and_then(|r| r.as_str()).map(String::from),
            resolved_hash,
            store_path: None,
            follows: None,
            last_modified: locked.get("lastModified")
                .and_then(serde_json::Value::as_i64)
                .and_then(|ts| DateTime::from_timestamp(ts, 0)),
        })
    }

    /// Get recommendations based on the analysis
    pub fn get_recommendations(&self, analysis: &FlakeLockAnalysis) -> Vec<String> {
        let mut recommendations = Vec::new();

        // Check for stale inputs
        if !analysis.stale_inputs.is_empty() {
            recommendations.push(format!(
                "⚠️  {} inputs haven't been updated in over {} days. Consider running 'nix flake update' for: {}",
                analysis.stale_inputs.len(),
                analysis.stale_inputs[0].days_stale,
                analysis.stale_inputs.iter()
                    .take(3)
                    .map(|s| &s.name)
                    .cloned()
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }

        // Check update frequency
        if analysis.update_patterns.avg_commits_per_month < 1.0 {
            recommendations.push(
                "📊 Updates are infrequent (less than once per month). Consider setting up automated dependency updates.".to_string()
            );
        } else if analysis.update_patterns.avg_commits_per_month > 10.0 {
            recommendations.push(
                "📊 Very frequent updates detected. Consider batching updates to reduce churn.".to_string()
            );
        }

        // Check for batch updates
        if analysis.update_patterns.batch_updates.len() > analysis.total_commits / 2 {
            recommendations.push(
                "✅ Good practice: Most updates are batched together, reducing the number of commits.".to_string()
            );
        }

        recommendations
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cim_domain_git::value_objects::AuthorInfo;

    fn create_test_commit(timestamp: DateTime<Utc>, inputs: Vec<(&str, &str)>) -> FlakeLockCommit {
        let mut nodes = serde_json::Map::new();
        
        for (name, rev) in inputs {
            nodes.insert(name.to_string(), serde_json::json!({
                "locked": {
                    "type": "github",
                    "owner": "NixOS",
                    "repo": name,
                    "rev": rev,
                    "lastModified": timestamp.timestamp()
                }
            }));
        }

        FlakeLockCommit {
            commit: CommitHash::new(&format!("{:0<40}", "abc123")).unwrap(),
            timestamp,
            message: "Update flake.lock".to_string(),
            author: AuthorInfo::new("Test User", "test@example.com"),
            lock_content: serde_json::json!({
                "nodes": nodes
            }),
        }
    }

    #[test]
    fn test_flake_lock_tracker() {
        let now = Utc::now();
        let commits = vec![
            create_test_commit(now - Duration::days(30), vec![("nixpkgs", "abc1")]),
            create_test_commit(now - Duration::days(20), vec![("nixpkgs", "abc2")]),
            create_test_commit(now - Duration::days(10), vec![("nixpkgs", "abc3"), ("flake-utils", "def1")]),
            create_test_commit(now, vec![("nixpkgs", "abc4"), ("flake-utils", "def2")]),
        ];

        let tracker = FlakeLockTracker::new(commits);
        let analysis = tracker.analyze(15);

        assert_eq!(analysis.total_commits, 4);
        assert_eq!(analysis.most_updated_inputs[0].0, "nixpkgs");
        assert_eq!(analysis.most_updated_inputs[0].1, 3);
        assert!(analysis.stale_inputs.is_empty()); // All updated recently
        assert_eq!(analysis.update_patterns.batch_updates.len(), 2); // Last 2 commits updated multiple inputs
    }
} 
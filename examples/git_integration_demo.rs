//! Example demonstrating Git-Nix integration functionality

use cim_domain_nix::git_integration::{
    GitFlakeService, 
    analyzer::GitNixAnalyzer, 
    flake_lock_tracker::FlakeLockTracker
};
use cim_domain_git::value_objects::CommitHash;
use std::path::Path;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("🔗 Git-Nix Integration Demo\n");

    // Parse command line arguments
    let args: Vec<String> = std::env::args().collect();
    let repo_path = args.get(1)
        .map(Path::new)
        .unwrap_or(Path::new("."));

    // Check if this is a Git repository
    if !repo_path.join(".git").exists() {
        eprintln!("❌ Not a Git repository: {}", repo_path.display());
        eprintln!("   Please run this from a Git repository with a flake.lock file");
        return Ok(());
    }

    // Check if flake.lock exists
    if !repo_path.join("flake.lock").exists() {
        eprintln!("❌ No flake.lock found in: {}", repo_path.display());
        eprintln!("   This demo requires a Nix flake with a flake.lock file");
        return Ok(());
    }

    println!("📂 Analyzing repository: {}\n", repo_path.display());

    // Create analyzer
    let analyzer = GitNixAnalyzer::new();

    // Analyze flake.lock history
    println!("📊 Analyzing flake.lock history...\n");
    
    match analyzer.get_flake_lock_history(repo_path, Some(10)).await {
        Ok(commits) => {
            if commits.is_empty() {
                println!("No flake.lock commits found in history");
            } else {
                println!("Found {commits.len(} flake.lock commits (showing up to 10):\n"));
                
                for (i, commit) in commits.iter().enumerate() {
                    println!("{i + 1}. {commit.timestamp.format("%Y-%m-%d %H:%M"} - {}"),
                        commit.commit.short()
                    );
                    println!("   Author: {commit.author}");
                    println!("   Message: {commit.message}");
                    
                    // Count inputs
                    if let Some(nodes) = commit.lock_content.get("nodes").and_then(|n| n.as_object()) {
                        let input_count = nodes.len() - 1; // Exclude "root"
                        println!("   Inputs: {input_count} dependencies locked");
                    }
                    println!();
                }

                // Create tracker for deeper analysis
                let tracker = FlakeLockTracker::new(commits);
                let analysis = tracker.analyze(30); // 30 days stale threshold

                println!("\n📈 Dependency Update Analysis:");
                println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
                
                println!("\n📊 Overall Statistics:");
                println!("   Total commits: {analysis.total_commits}");
                if let Some(span) = analysis.time_span {
                    println!("   Time span: {span.num_days(} days"));
                }
                println!("   Avg commits/month: {:.1}", analysis.update_patterns.avg_commits_per_month);

                println!("\n🔥 Most Updated Inputs:");
                for (name, count) in analysis.most_updated_inputs.iter().take(5) {
                    println!("   {name} - {count} updates");
                }

                if !analysis.stale_inputs.is_empty() {
                    println!("\n⚠️  Stale Inputs (>30 days):");
                    for stale in &analysis.stale_inputs {
                        println!("   {stale.name} - {stale.days_stale} days since update");
                    }
                }

                println!("\n📅 Update Patterns:");
                if let Some(day) = analysis.update_patterns.most_active_day {
                    let days = ["Sunday", "Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday"];
                    println!("   Most active day: {days[day as usize]}");
                }
                if let Some(hour) = analysis.update_patterns.most_active_hour {
                    println!("   Most active hour: {hour}:00");
                }
                
                if !analysis.update_patterns.batch_updates.is_empty() {
                    println!("\n🎯 Batch Updates Detected:");
                    for batch in analysis.update_patterns.batch_updates.iter().take(3) {
                        println!("   {batch.timestamp.format("%Y-%m-%d"} - {} inputs updated together"),
                            batch.inputs_updated
                        );
                        println!("     {batch.input_names.join(", "}"));
                    }
                }

                // Get recommendations
                let recommendations = tracker.get_recommendations(&analysis);
                if !recommendations.is_empty() {
                    println!("\n💡 Recommendations:");
                    for rec in recommendations {
                        println!("   {rec}");
                    }
                }
            }
        }
        Err(e) => {
            eprintln!("Error analyzing flake.lock history: {e}");
        }
    }

    // Analyze dependency changes between commits
    if args.len() >= 4 && args[2] == "--compare" {
        let from_commit = args[3].as_str();
        let to_commit = args.get(4).map(|s| s.as_str()).unwrap_or("HEAD");

        println!("\n\n🔄 Comparing dependency changes:");
        println!("   From: {from_commit}");
        println!("   To: {to_commit}\n");

        match (CommitHash::new(from_commit), CommitHash::new(to_commit)) {
            (Ok(from), Ok(to)) => {
                match analyzer.analyze_dependency_changes(repo_path, &from, &to).await {
                    Ok(changes) => {
                        if !changes.added.is_empty() {
                            println!("➕ Added inputs:");
                            for input in &changes.added {
                                println!("   {input.name} ({input.url})");
                            }
                        }

                        if !changes.removed.is_empty() {
                            println!("\n➖ Removed inputs:");
                            for input in &changes.removed {
                                println!("   {input.name} ({input.url})");
                            }
                        }

                        if !changes.updated.is_empty() {
                            println!("\n🔄 Updated inputs:");
                            for (old, new) in &changes.updated {
                                println!("   {new.name}");
                                if let (Some(old_hash), Some(new_hash)) = (&old.resolved_hash, &new.resolved_hash) {
                                    println!("     {old_hash.short(} → {}"), new_hash.short());
                                }
                            }
                        }

                        if changes.added.is_empty() && changes.removed.is_empty() && changes.updated.is_empty() {
                            println!("No dependency changes between these commits");
                        }
                    }
                    Err(e) => eprintln!("Error analyzing changes: {e}"),
                }
            }
            _ => eprintln!("Invalid commit hashes provided"),
        }
    }

    // Analyze Nix file changes
    println!("\n\n📝 Recent Nix File Changes:");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━");
    
    match analyzer.analyze_nix_file_changes(repo_path, None, Some(10)).await {
        Ok(changes) => {
            if changes.is_empty() {
                println!("No Nix file changes found");
            } else {
                for change in changes.iter().take(5) {
                    println!("\n{change.timestamp.format("%Y-%m-%d %H:%M"} - {}"),
                        change.commit.short()
                    );
                    println!("Author: {change.author}");
                    println!("Files changed:");
                    for file in &change.files {
                        let symbol = match file.change_type {
                            cim_domain_nix::git_integration::analyzer::FileChangeType::Added => "➕",
                            cim_domain_nix::git_integration::analyzer::FileChangeType::Modified => "📝",
                            cim_domain_nix::git_integration::analyzer::FileChangeType::Deleted => "🗑️",
                            cim_domain_nix::git_integration::analyzer::FileChangeType::Renamed => "📋",
                        };
                        println!("  {symbol} {file.path}");
                    }
                }
            }
        }
        Err(e) => eprintln!("Error analyzing Nix file changes: {e}"),
    }

    println!("\n✅ Analysis complete!");
    
    Ok(())
} 
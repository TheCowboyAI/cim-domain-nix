//! Integration tests for the Nix analyzer

use cim_domain_nix::analyzer::{NixAnalyzer, AnalyzerConfig};
use std::path::Path;

#[tokio::test]
async fn test_analyzer_detects_security_issues() {
    // Create analyzer
    let config = AnalyzerConfig {
        parallel_parsing: false,
        max_files: Some(10),
        follow_symlinks: false,
        ignore_patterns: vec![".git".to_string()],
    };
    
    let mut analyzer = NixAnalyzer::with_config(config);
    
    // Analyze the test files
    let test_path = Path::new("examples/test_nix_files");
    let report = analyzer.analyze_repository(test_path).await.unwrap();
    
    // Verify security issues were found
    assert!(!report.security_issues.is_empty(), "Should find security issues");
    
    // Check for specific issue types
    let has_insecure_fetcher = report.security_issues.iter()
        .any(|issue| matches!(issue.issue_type, 
            cim_domain_nix::analyzer::security::SecurityIssueType::InsecureFetcher));
    assert!(has_insecure_fetcher, "Should detect insecure fetchers");
    
    let has_weak_hash = report.security_issues.iter()
        .any(|issue| matches!(issue.issue_type,
            cim_domain_nix::analyzer::security::SecurityIssueType::WeakHash));
    assert!(has_weak_hash, "Should detect weak hashes");
}

#[tokio::test]
async fn test_analyzer_detects_performance_issues() {
    let config = AnalyzerConfig::default();
    let mut analyzer = NixAnalyzer::with_config(config);
    
    let test_path = Path::new("examples/test_nix_files");
    let report = analyzer.analyze_repository(test_path).await.unwrap();
    
    // Verify performance issues were found
    assert!(!report.performance_issues.is_empty(), "Should find performance issues");
    
    // Check for string concatenation issues
    let has_string_concat = report.performance_issues.iter()
        .any(|issue| matches!(issue.issue_type,
            cim_domain_nix::analyzer::performance::PerformanceIssueType::InefficientStringConcat));
    assert!(has_string_concat, "Should detect inefficient string concatenations");
}

#[tokio::test]
async fn test_analyzer_detects_dead_code() {
    let config = AnalyzerConfig::default();
    let mut analyzer = NixAnalyzer::with_config(config);
    
    let test_path = Path::new("examples/test_nix_files");
    let report = analyzer.analyze_repository(test_path).await.unwrap();
    
    // Verify dead code was found
    assert!(!report.dead_code.is_empty(), "Should find dead code");
    
    // Check for unreachable code
    let has_unreachable = report.dead_code.iter()
        .any(|dead| matches!(dead.code_type,
            cim_domain_nix::analyzer::dead_code::DeadCodeType::UnreachableCode));
    assert!(has_unreachable, "Should detect unreachable code");
} 
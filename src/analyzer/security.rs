//! Security analysis for Nix files
//!
//! This module detects security issues such as insecure fetchers,
//! impure functions, unsafe derivations, and missing hashes.

use crate::parser::NixFile;
use crate::Result;
use rnix::{SyntaxKind, SyntaxNode};
use serde;

/// Security issue severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Serialize)]
pub enum Severity {
    /// Low severity - best practice violation
    Low,
    /// Medium severity - potential security issue
    Medium,
    /// High severity - definite security risk
    High,
    /// Critical severity - immediate security threat
    Critical,
}

/// A security issue found in a Nix file
#[derive(Debug, Clone, serde::Serialize)]
pub struct SecurityIssue {
    /// Type of security issue
    pub issue_type: SecurityIssueType,
    /// Severity level
    pub severity: Severity,
    /// Description of the issue
    pub description: String,
    /// File where the issue was found
    pub file: Option<String>,
    /// Line number (if available)
    pub line: Option<usize>,
    /// Column number (if available)
    pub column: Option<usize>,
    /// Suggested fix
    pub suggestion: Option<String>,
}

/// Types of security issues
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub enum SecurityIssueType {
    /// Using fetchurl without a hash
    InsecureFetcher,
    /// Using impure functions like builtins.currentTime
    ImpureFunction,
    /// Derivation with allowUnfree = true
    UnfreeAllowed,
    /// Using fixed-output derivation with weak hash
    WeakHash,
    /// Executing arbitrary code
    ArbitraryCodeExecution,
    /// Insecure network access
    InsecureNetworkAccess,
    /// Missing source verification
    MissingSourceVerification,
    /// World-writable permissions
    InsecurePermissions,
    /// Using deprecated security features
    DeprecatedSecurity,
}

/// Security analyzer for Nix files
pub struct SecurityAnalyzer;

impl SecurityAnalyzer {
    /// Analyze files for security issues
    pub fn analyze(files: &[NixFile]) -> Result<Vec<SecurityIssue>> {
        let mut issues = Vec::new();

        for file in files {
            issues.extend(Self::analyze_file(file)?);
        }

        // Sort by severity (highest first) then by file
        issues.sort_by(|a, b| {
            b.severity.cmp(&a.severity)
                .then_with(|| a.file.cmp(&b.file))
        });

        Ok(issues)
    }

    /// Analyze a single file
    fn analyze_file(file: &NixFile) -> Result<Vec<SecurityIssue>> {
        let mut issues = Vec::new();

        // Run various security checks
        issues.extend(Self::check_insecure_fetchers(&file.ast, &file.source)?);
        issues.extend(Self::check_impure_functions(&file.ast, &file.source)?);
        issues.extend(Self::check_unsafe_derivations(&file.ast, &file.source)?);
        issues.extend(Self::check_fixed_output_hashes(&file.ast, &file.source)?);
        issues.extend(Self::check_arbitrary_code_execution(&file.ast, &file.source)?);
        issues.extend(Self::check_network_security(&file.ast, &file.source)?);

        Ok(issues)
    }

    /// Check for insecure fetchers (fetchurl without hash, etc.)
    fn check_insecure_fetchers(
        node: &SyntaxNode,
        file: &Option<std::path::PathBuf>,
    ) -> Result<Vec<SecurityIssue>> {
        let mut issues = Vec::new();

        // Look for fetch* functions
        if node.kind() == SyntaxKind::NODE_APPLY {
            let text = node.text().to_string();
            
            // Check various fetchers
            let fetchers = [
                "fetchurl",
                "fetchTarball",
                "fetchGit",
                "fetchFromGitHub",
                "fetchzip",
                "fetchpatch",
            ];

            for fetcher in &fetchers {
                if text.contains(fetcher) {
                    // Check if it has a hash argument
                    let has_hash = Self::has_hash_argument(node);
                    
                    if !has_hash {
                        issues.push(SecurityIssue {
                            issue_type: SecurityIssueType::InsecureFetcher,
                            severity: Severity::High,
                            description: format!(
                                "Using {} without a hash allows arbitrary code changes",
                                fetcher
                            ),
                            file: file.as_ref().map(|p| p.display().to_string()),
                            line: None, // TODO: Extract line info
                            column: None,
                            suggestion: Some(format!(
                                "Add a sha256 or hash attribute to the {} call",
                                fetcher
                            )),
                        });
                    }
                }
            }
        }

        // Recurse
        for child in node.children() {
            issues.extend(Self::check_insecure_fetchers(&child, file)?);
        }

        Ok(issues)
    }

    /// Check for impure functions
    fn check_impure_functions(
        node: &SyntaxNode,
        file: &Option<std::path::PathBuf>,
    ) -> Result<Vec<SecurityIssue>> {
        let mut issues = Vec::new();

        let text = node.text().to_string();
        
        // List of impure functions
        let impure_functions = [
            ("builtins.currentTime", "Returns non-deterministic current time"),
            ("builtins.currentSystem", "Returns system-dependent value"),
            ("builtins.getEnv", "Reads from environment variables"),
            ("builtins.readFile", "Reads files at evaluation time"),
            ("builtins.readDir", "Reads directory contents at evaluation time"),
            ("builtins.fetchurl", "Fetches from network without sandboxing"),
        ];

        for (func, desc) in &impure_functions {
            if text.contains(func) {
                issues.push(SecurityIssue {
                    issue_type: SecurityIssueType::ImpureFunction,
                    severity: Severity::Medium,
                    description: format!("{}: {}", func, desc),
                    file: file.as_ref().map(|p| p.display().to_string()),
                    line: None,
                    column: None,
                    suggestion: Some("Consider using pure alternatives or making the evaluation deterministic".to_string()),
                });
            }
        }

        // Recurse
        for child in node.children() {
            issues.extend(Self::check_impure_functions(&child, file)?);
        }

        Ok(issues)
    }

    /// Check for unsafe derivations
    fn check_unsafe_derivations(
        node: &SyntaxNode,
        file: &Option<std::path::PathBuf>,
    ) -> Result<Vec<SecurityIssue>> {
        let mut issues = Vec::new();

        let text = node.text().to_string();

        // Check for allowUnfree
        if text.contains("allowUnfree = true") || text.contains("allowUnfree=true") {
            issues.push(SecurityIssue {
                issue_type: SecurityIssueType::UnfreeAllowed,
                severity: Severity::Low,
                description: "Allowing unfree packages may include proprietary code".to_string(),
                file: file.as_ref().map(|p| p.display().to_string()),
                line: None,
                column: None,
                suggestion: Some("Only allow unfree packages that are explicitly needed".to_string()),
            });
        }

        // Check for allowInsecure
        if text.contains("allowInsecure = true") || text.contains("allowInsecure=true") {
            issues.push(SecurityIssue {
                issue_type: SecurityIssueType::InsecureNetworkAccess,
                severity: Severity::High,
                description: "Allowing insecure packages exposes known vulnerabilities".to_string(),
                file: file.as_ref().map(|p| p.display().to_string()),
                line: None,
                column: None,
                suggestion: Some("Update to secure versions or explicitly accept the risk".to_string()),
            });
        }

        // Check for sandbox = false
        if text.contains("sandbox = false") || text.contains("sandbox=false") {
            issues.push(SecurityIssue {
                issue_type: SecurityIssueType::ArbitraryCodeExecution,
                severity: Severity::Critical,
                description: "Disabling sandbox allows builds to access the network and filesystem".to_string(),
                file: file.as_ref().map(|p| p.display().to_string()),
                line: None,
                column: None,
                suggestion: Some("Keep sandbox enabled for security".to_string()),
            });
        }

        // Recurse
        for child in node.children() {
            issues.extend(Self::check_unsafe_derivations(&child, file)?);
        }

        Ok(issues)
    }

    /// Check for weak or missing fixed-output hashes
    fn check_fixed_output_hashes(
        node: &SyntaxNode,
        file: &Option<std::path::PathBuf>,
    ) -> Result<Vec<SecurityIssue>> {
        let mut issues = Vec::new();

        let text = node.text().to_string();

        // Check for MD5 hashes
        if text.contains("md5 = ") || text.contains("md5=") {
            issues.push(SecurityIssue {
                issue_type: SecurityIssueType::WeakHash,
                severity: Severity::High,
                description: "MD5 is cryptographically broken and should not be used".to_string(),
                file: file.as_ref().map(|p| p.display().to_string()),
                line: None,
                column: None,
                suggestion: Some("Use sha256 or sha512 instead".to_string()),
            });
        }

        // Check for SHA1 hashes
        if text.contains("sha1 = ") || text.contains("sha1=") {
            issues.push(SecurityIssue {
                issue_type: SecurityIssueType::WeakHash,
                severity: Severity::Medium,
                description: "SHA1 is deprecated for security purposes".to_string(),
                file: file.as_ref().map(|p| p.display().to_string()),
                line: None,
                column: None,
                suggestion: Some("Use sha256 or sha512 instead".to_string()),
            });
        }

        // Recurse
        for child in node.children() {
            issues.extend(Self::check_fixed_output_hashes(&child, file)?);
        }

        Ok(issues)
    }

    /// Check for arbitrary code execution risks
    fn check_arbitrary_code_execution(
        node: &SyntaxNode,
        file: &Option<std::path::PathBuf>,
    ) -> Result<Vec<SecurityIssue>> {
        let mut issues = Vec::new();

        let text = node.text().to_string();

        // Check for eval usage
        if text.contains("builtins.exec") {
            issues.push(SecurityIssue {
                issue_type: SecurityIssueType::ArbitraryCodeExecution,
                severity: Severity::Critical,
                description: "Using builtins.exec allows arbitrary code execution".to_string(),
                file: file.as_ref().map(|p| p.display().to_string()),
                line: None,
                column: None,
                suggestion: Some("Avoid executing arbitrary code during evaluation".to_string()),
            });
        }

        // Check for import from derivation (IFD)
        if text.contains("import (") && text.contains("mkDerivation") {
            issues.push(SecurityIssue {
                issue_type: SecurityIssueType::ArbitraryCodeExecution,
                severity: Severity::Medium,
                description: "Import from derivation (IFD) can execute code during evaluation".to_string(),
                file: file.as_ref().map(|p| p.display().to_string()),
                line: None,
                column: None,
                suggestion: Some("Consider alternatives to IFD for better security and performance".to_string()),
            });
        }

        // Recurse
        for child in node.children() {
            issues.extend(Self::check_arbitrary_code_execution(&child, file)?);
        }

        Ok(issues)
    }

    /// Check for network security issues
    fn check_network_security(
        node: &SyntaxNode,
        file: &Option<std::path::PathBuf>,
    ) -> Result<Vec<SecurityIssue>> {
        let mut issues = Vec::new();

        let text = node.text().to_string();

        // Check for HTTP URLs (not HTTPS)
        if text.contains("http://") && !text.contains("http://localhost") && !text.contains("http://127.0.0.1") {
            issues.push(SecurityIssue {
                issue_type: SecurityIssueType::InsecureNetworkAccess,
                severity: Severity::Medium,
                description: "Using HTTP instead of HTTPS allows man-in-the-middle attacks".to_string(),
                file: file.as_ref().map(|p| p.display().to_string()),
                line: None,
                column: None,
                suggestion: Some("Use HTTPS URLs for secure communication".to_string()),
            });
        }

        // Check for git:// protocol
        if text.contains("git://") {
            issues.push(SecurityIssue {
                issue_type: SecurityIssueType::InsecureNetworkAccess,
                severity: Severity::Medium,
                description: "Git protocol is unencrypted and unauthenticated".to_string(),
                file: file.as_ref().map(|p| p.display().to_string()),
                line: None,
                column: None,
                suggestion: Some("Use https:// or ssh:// for Git URLs".to_string()),
            });
        }

        // Recurse
        for child in node.children() {
            issues.extend(Self::check_network_security(&child, file)?);
        }

        Ok(issues)
    }

    /// Check if a fetch call has a hash argument
    fn has_hash_argument(node: &SyntaxNode) -> bool {
        let text = node.text().to_string();
        
        // Look for hash-related attributes
        text.contains("sha256") || 
        text.contains("sha512") || 
        text.contains("hash =") ||
        text.contains("outputHash")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_insecure_fetcher() {
        let content = r#"
        {
            src = fetchurl {
                url = "https://example.com/source.tar.gz";
            };
        }
        "#;

        let file = NixFile::parse_string(content.to_string(), None).unwrap();
        let issues = SecurityAnalyzer::analyze(&[file]).unwrap();

        assert!(!issues.is_empty());
        assert_eq!(issues[0].issue_type, SecurityIssueType::InsecureFetcher);
        assert_eq!(issues[0].severity, Severity::High);
    }

    #[test]
    fn test_detect_weak_hash() {
        let content = r#"
        {
            src = fetchurl {
                url = "https://example.com/source.tar.gz";
                md5 = "d41d8cd98f00b204e9800998ecf8427e";
            };
        }
        "#;

        let file = NixFile::parse_string(content.to_string(), None).unwrap();
        let issues = SecurityAnalyzer::analyze(&[file]).unwrap();

        let weak_hash_issues: Vec<_> = issues.iter()
            .filter(|i| i.issue_type == SecurityIssueType::WeakHash)
            .collect();

        assert!(!weak_hash_issues.is_empty());
        assert_eq!(weak_hash_issues[0].severity, Severity::High);
    }

    #[test]
    fn test_secure_fetcher() {
        let content = r#"
        {
            src = fetchurl {
                url = "https://example.com/source.tar.gz";
                sha256 = "0000000000000000000000000000000000000000000000000000000000000000";
            };
        }
        "#;

        let file = NixFile::parse_string(content.to_string(), None).unwrap();
        let issues = SecurityAnalyzer::analyze(&[file]).unwrap();

        // Should not have insecure fetcher issues
        let fetcher_issues: Vec<_> = issues.iter()
            .filter(|i| i.issue_type == SecurityIssueType::InsecureFetcher)
            .collect();

        assert!(fetcher_issues.is_empty());
    }
} 
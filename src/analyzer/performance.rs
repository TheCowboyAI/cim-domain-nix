//! Performance analysis for Nix files
//!
//! This module detects performance issues such as IFD (Import From Derivation),
//! excessive recursion, large lists, and inefficient patterns.

use crate::parser::NixFile;
use crate::Result;
use rnix::{SyntaxKind, SyntaxNode};

/// A performance issue found in a Nix file
#[derive(Debug, Clone, serde::Serialize)]
pub struct PerformanceIssue {
    /// Type of performance issue
    pub issue_type: PerformanceIssueType,
    /// Impact level
    pub impact: ImpactLevel,
    /// Description of the issue
    pub description: String,
    /// File where the issue was found
    pub file: Option<String>,
    /// Line number (if available)
    pub line: Option<usize>,
    /// Estimated performance cost
    pub cost_estimate: Option<String>,
    /// Suggested optimization
    pub suggestion: Option<String>,
}

/// Types of performance issues
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub enum PerformanceIssueType {
    /// Import from derivation
    ImportFromDerivation,
    /// Excessive recursion
    ExcessiveRecursion,
    /// Large list operations
    LargeListOperation,
    /// Inefficient string concatenation
    InefficientStringConcat,
    /// Repeated evaluation
    RepeatedEvaluation,
    /// Deep attribute access
    DeepAttributeAccess,
    /// Unnecessary imports
    UnnecessaryImports,
    /// Complex overlays
    ComplexOverlay,
    /// Inefficient pattern
    InefficientPattern,
}

/// Impact level of performance issues
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Serialize)]
pub enum ImpactLevel {
    /// Minor impact
    Low,
    /// Moderate impact
    Medium,
    /// Significant impact
    High,
    /// Severe impact
    Critical,
}

/// Performance analyzer for Nix files
pub struct PerformanceAnalyzer;

impl PerformanceAnalyzer {
    /// Analyze files for performance issues
    pub fn analyze(files: &[NixFile]) -> Result<Vec<PerformanceIssue>> {
        let mut issues = Vec::new();

        for file in files {
            issues.extend(Self::analyze_file(file)?);
        }

        // Sort by impact (highest first)
        issues.sort_by(|a, b| b.impact.cmp(&a.impact).then_with(|| a.file.cmp(&b.file)));

        Ok(issues)
    }

    /// Analyze a single file
    fn analyze_file(file: &NixFile) -> Result<Vec<PerformanceIssue>> {
        let mut issues = Vec::new();

        // Run various performance checks
        issues.extend(Self::check_import_from_derivation(&file.ast, &file.source)?);
        issues.extend(Self::check_excessive_recursion(&file.ast, &file.source)?);
        issues.extend(Self::check_large_list_operations(&file.ast, &file.source)?);
        issues.extend(Self::check_string_concatenation(&file.ast, &file.source)?);
        issues.extend(Self::check_repeated_evaluation(&file.ast, &file.source)?);
        issues.extend(Self::check_deep_attribute_access(&file.ast, &file.source)?);

        Ok(issues)
    }

    /// Check for Import From Derivation (IFD)
    fn check_import_from_derivation(
        node: &SyntaxNode,
        file: &Option<std::path::PathBuf>,
    ) -> Result<Vec<PerformanceIssue>> {
        let mut issues = Vec::new();

        // Look for import expressions
        if node.kind() == SyntaxKind::NODE_APPLY {
            let text = node.text().to_string();

            // Check if this is an import call
            if text.contains("import") {
                // Check if the import argument contains derivation-related functions
                for child in node.children() {
                    if Self::contains_derivation(&child) {
                        issues.push(PerformanceIssue {
                            issue_type: PerformanceIssueType::ImportFromDerivation,
                            impact: ImpactLevel::High,
                            description: "Import from derivation (IFD) forces evaluation during build".to_string(),
                            file: file.as_ref().map(|p| p.display().to_string()),
                            line: None,
                            cost_estimate: Some("Can add minutes to evaluation time".to_string()),
                            suggestion: Some("Pre-build and commit the imported file, or use a different approach".to_string()),
                        });
                        break;
                    }
                }
            }
        }

        // Also check for parenthesized expressions that might contain import + derivation
        if node.kind() == SyntaxKind::NODE_PAREN {
            let text = node.text().to_string();
            if text.contains("import") && text.contains("runCommand") {
                issues.push(PerformanceIssue {
                    issue_type: PerformanceIssueType::ImportFromDerivation,
                    impact: ImpactLevel::High,
                    description: "Import from derivation (IFD) forces evaluation during build"
                        .to_string(),
                    file: file.as_ref().map(|p| p.display().to_string()),
                    line: None,
                    cost_estimate: Some("Can add minutes to evaluation time".to_string()),
                    suggestion: Some(
                        "Pre-build and commit the imported file, or use a different approach"
                            .to_string(),
                    ),
                });
            }
        }

        // Recurse
        for child in node.children() {
            issues.extend(Self::check_import_from_derivation(&child, file)?);
        }

        Ok(issues)
    }

    /// Check for excessive recursion
    fn check_excessive_recursion(
        node: &SyntaxNode,
        file: &Option<std::path::PathBuf>,
    ) -> Result<Vec<PerformanceIssue>> {
        let mut issues = Vec::new();

        // Count recursion depth
        let depth = Self::measure_recursion_depth(node);

        if depth > 10 {
            let impact = if depth > 20 {
                ImpactLevel::Critical
            } else if depth > 15 {
                ImpactLevel::High
            } else {
                ImpactLevel::Medium
            };

            issues.push(PerformanceIssue {
                issue_type: PerformanceIssueType::ExcessiveRecursion,
                impact,
                description: format!("Deep recursion detected (depth: {depth})"),
                file: file.as_ref().map(|p| p.display().to_string()),
                line: None,
                cost_estimate: Some(format!("Exponential evaluation cost at depth {depth}")),
                suggestion: Some(
                    "Consider using builtins.foldl' or iterative approaches".to_string(),
                ),
            });
        }

        Ok(issues)
    }

    /// Check for large list operations
    fn check_large_list_operations(
        node: &SyntaxNode,
        file: &Option<std::path::PathBuf>,
    ) -> Result<Vec<PerformanceIssue>> {
        let mut issues = Vec::new();

        let text = node.text().to_string();

        // Check for inefficient list operations
        let inefficient_patterns = [
            ("++ [", "Appending single elements repeatedly"),
            ("lib.concatLists", "Consider using builtins.concatLists"),
            ("map (x: map", "Nested map operations"),
            ("filter (x: filter", "Nested filter operations"),
        ];

        for (pattern, desc) in &inefficient_patterns {
            if text.contains(pattern) {
                issues.push(PerformanceIssue {
                    issue_type: PerformanceIssueType::LargeListOperation,
                    impact: ImpactLevel::Medium,
                    description: (*desc).to_string(),
                    file: file.as_ref().map(|p| p.display().to_string()),
                    line: None,
                    cost_estimate: Some("O(n²) complexity for large lists".to_string()),
                    suggestion: Some("Use more efficient list building patterns".to_string()),
                });
            }
        }

        // Check for list comprehensions that could be optimized
        if text.contains("lib.flatten") && text.contains("map") {
            issues.push(PerformanceIssue {
                issue_type: PerformanceIssueType::LargeListOperation,
                impact: ImpactLevel::Low,
                description: "Consider using concatMap instead of map + flatten".to_string(),
                file: file.as_ref().map(|p| p.display().to_string()),
                line: None,
                cost_estimate: Some("Double traversal of list".to_string()),
                suggestion: Some("Use lib.concatMap for better performance".to_string()),
            });
        }

        // Recurse
        for child in node.children() {
            issues.extend(Self::check_large_list_operations(&child, file)?);
        }

        Ok(issues)
    }

    /// Check for inefficient string concatenation
    fn check_string_concatenation(
        node: &SyntaxNode,
        file: &Option<std::path::PathBuf>,
    ) -> Result<Vec<PerformanceIssue>> {
        let mut issues = Vec::new();

        // Count string concatenations in a single expression
        let concat_count = Self::count_string_concatenations(node);

        if concat_count > 5 {
            let impact = if concat_count > 20 {
                ImpactLevel::High
            } else if concat_count > 10 {
                ImpactLevel::Medium
            } else {
                ImpactLevel::Low
            };

            issues.push(PerformanceIssue {
                issue_type: PerformanceIssueType::InefficientStringConcat,
                impact,
                description: format!("Many string concatenations ({concat_count} found)"),
                file: file.as_ref().map(|p| p.display().to_string()),
                line: None,
                cost_estimate: Some("O(n²) string building complexity".to_string()),
                suggestion: Some(
                    "Use string interpolation or builtins.concatStringsSep".to_string(),
                ),
            });
        }

        // Recurse
        for child in node.children() {
            issues.extend(Self::check_string_concatenation(&child, file)?);
        }

        Ok(issues)
    }

    /// Check for repeated evaluation
    fn check_repeated_evaluation(
        node: &SyntaxNode,
        file: &Option<std::path::PathBuf>,
    ) -> Result<Vec<PerformanceIssue>> {
        let mut issues = Vec::new();

        // Look for let expressions that could be hoisted
        if node.kind() == SyntaxKind::NODE_LET_IN {
            // Check if the let binding is inside a function that's called multiple times
            if Self::is_inside_repeated_context(node) {
                issues.push(PerformanceIssue {
                    issue_type: PerformanceIssueType::RepeatedEvaluation,
                    impact: ImpactLevel::Medium,
                    description: "Let binding inside frequently called function".to_string(),
                    file: file.as_ref().map(|p| p.display().to_string()),
                    line: None,
                    cost_estimate: Some("Evaluated on every function call".to_string()),
                    suggestion: Some(
                        "Consider hoisting constant computations outside the function".to_string(),
                    ),
                });
            }
        }

        // Check for repeated imports
        let text = node.text().to_string();
        if text.matches("import ").count() > 3 {
            issues.push(PerformanceIssue {
                issue_type: PerformanceIssueType::UnnecessaryImports,
                impact: ImpactLevel::Low,
                description: "Multiple imports of potentially the same file".to_string(),
                file: file.as_ref().map(|p| p.display().to_string()),
                line: None,
                cost_estimate: Some("Each import is parsed separately".to_string()),
                suggestion: Some("Import once and reuse the result".to_string()),
            });
        }

        // Recurse
        for child in node.children() {
            issues.extend(Self::check_repeated_evaluation(&child, file)?);
        }

        Ok(issues)
    }

    /// Check for deep attribute access patterns
    fn check_deep_attribute_access(
        node: &SyntaxNode,
        file: &Option<std::path::PathBuf>,
    ) -> Result<Vec<PerformanceIssue>> {
        let mut issues = Vec::new();

        // Count depth of attribute access
        let depth = Self::count_attribute_depth(node);

        if depth > 5 {
            let impact = if depth > 10 {
                ImpactLevel::Medium
            } else {
                ImpactLevel::Low
            };

            issues.push(PerformanceIssue {
                issue_type: PerformanceIssueType::DeepAttributeAccess,
                impact,
                description: format!("Deep attribute access (depth: {depth})"),
                file: file.as_ref().map(|p| p.display().to_string()),
                line: None,
                cost_estimate: Some("Each level requires lookup".to_string()),
                suggestion: Some("Consider destructuring or intermediate bindings".to_string()),
            });
        }

        // Recurse
        for child in node.children() {
            issues.extend(Self::check_deep_attribute_access(&child, file)?);
        }

        Ok(issues)
    }

    /// Helper: Check if node contains a derivation
    fn contains_derivation(node: &SyntaxNode) -> bool {
        let text = node.text().to_string();
        text.contains("mkDerivation")
            || text.contains("buildPackage")
            || text.contains("stdenv.mkDerivation")
            || text.contains("runCommand")
            || text.contains("writeText")
            || text.contains("writeFile")
    }

    /// Helper: Measure recursion depth
    fn measure_recursion_depth(node: &SyntaxNode) -> usize {
        // Simple heuristic: count nested function applications
        let mut max_depth = 0;
        let mut current_depth = 0;

        fn traverse(node: &SyntaxNode, depth: &mut usize, max: &mut usize) {
            if node.kind() == SyntaxKind::NODE_APPLY {
                *depth += 1;
                *max = (*max).max(*depth);
            }

            for child in node.children() {
                traverse(&child, depth, max);
            }

            if node.kind() == SyntaxKind::NODE_APPLY {
                *depth -= 1;
            }
        }

        traverse(node, &mut current_depth, &mut max_depth);
        max_depth
    }

    /// Helper: Count string concatenations
    fn count_string_concatenations(node: &SyntaxNode) -> usize {
        node.text().to_string().matches(" + ").count()
            + node.text().to_string().matches("++").count()
    }

    /// Helper: Check if inside repeated context
    fn is_inside_repeated_context(node: &SyntaxNode) -> bool {
        // Walk up the tree to see if we're inside a map, filter, or similar
        let mut current = node.clone();

        while let Some(parent) = current.parent() {
            let parent_text = parent.text().to_string();
            if parent_text.contains("map ")
                || parent_text.contains("filter ")
                || parent_text.contains("fold")
            {
                return true;
            }
            current = parent;
        }

        false
    }

    /// Helper: Count attribute access depth
    fn count_attribute_depth(node: &SyntaxNode) -> usize {
        // Count consecutive dots in attribute access
        let text = node.text().to_string();
        text.split_whitespace()
            .filter(|s| s.contains('.'))
            .map(|s| s.matches('.').count())
            .max()
            .unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_ifd() {
        let content = r#"
        let
          generatedConfig = import (pkgs.runCommand "config" {} ''
            echo '{ foo = "bar"; }' > $out
          '');
        in generatedConfig
        "#;

        let file = NixFile::parse_string(content.to_string(), None).unwrap();
        let issues = PerformanceAnalyzer::analyze(&[file]).unwrap();

        let ifd_issues: Vec<_> = issues
            .iter()
            .filter(|i| i.issue_type == PerformanceIssueType::ImportFromDerivation)
            .collect();

        assert!(!ifd_issues.is_empty());
        assert_eq!(ifd_issues[0].impact, ImpactLevel::High);
    }

    #[test]
    fn test_detect_inefficient_list_ops() {
        let content = r#"
        let
          list = [ 1 2 3 ];
          result = list ++ [ 4 ] ++ [ 5 ] ++ [ 6 ];
        in result
        "#;

        let file = NixFile::parse_string(content.to_string(), None).unwrap();
        let issues = PerformanceAnalyzer::analyze(&[file]).unwrap();

        assert!(issues
            .iter()
            .any(|i| i.issue_type == PerformanceIssueType::LargeListOperation));
    }

    #[test]
    fn test_detect_deep_attributes() {
        let content = r#"
        {
          value = config.services.nginx.virtualHosts.example.locations.root.extraConfig;
        }
        "#;

        let file = NixFile::parse_string(content.to_string(), None).unwrap();
        let issues = PerformanceAnalyzer::analyze(&[file]).unwrap();

        let deep_access: Vec<_> = issues
            .iter()
            .filter(|i| i.issue_type == PerformanceIssueType::DeepAttributeAccess)
            .collect();

        assert!(!deep_access.is_empty());
    }
}

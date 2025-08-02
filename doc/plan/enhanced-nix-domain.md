# Enhanced Nix Domain Implementation Plan

## Overview

This document outlines the comprehensive enhancement plan for the Nix domain in CIM, transforming it from a basic wrapper around Nix commands into a sophisticated domain that provides deep analysis, optimization, and integration capabilities.

## Current Status

The Nix domain currently provides:
- Basic flake management (create, update, build)
- Simple command execution wrappers
- Minimal error handling
- Basic value objects for Nix concepts

## Completed Phases

✅ **Phase 1: Core Parser Infrastructure** - AST parsing with rnix, expression evaluation
✅ **Phase 2: Advanced Formatter Integration** - Support for nixpkgs-fmt, alejandra, nixfmt
✅ **Phase 3: Analysis Framework** - Complete dependency, security, performance, and dead code analysis
✅ **Phase 4: Git Integration** - Flake.lock history tracking and Nix file change analysis
✅ **Phase 5: NATS Integration** - Full event streaming with 46 mapped subjects
✅ **Phase 6: Network Integration** - Auto NixOS generation from network topology
⏳ **Phase 7: Home Manager Integration** - Partially complete (20%)
❌ **Phase 8: LSP Implementation** - Not started
❌ **Phase 9: Cross-Domain Integration** - Not started
❌ **Phase 10: Production Hardening** - Not started

## Vision

The Nix domain will be the premier module for programmatically working with Nix files, providing:
- **Complete AST manipulation** for all Nix file types
- **Deep analysis** of flake dependencies and structure
- **Workflow generation** from Nix configurations
- **Git integration** for analyzing Nix files across repositories
- **NixOS-specific** understanding of modules, options, and configurations
- **Performance** to handle massive monorepos like nixpkgs

## Core Requirements

### 1. Comprehensive Nix File Support
- **Flakes**: Full manipulation, analysis, and generation
- **NixOS Modules**: Understanding imports, options, config
- **Overlays**: Analysis and modification
- **Derivations**: Parse and understand package definitions
- **Home Manager**: Support for home.nix files
- **nix-darwin**: Support for darwin configurations

### 2. Advanced Analysis Capabilities
- **Dependency graphs**: Visualize flake input dependencies
- **Option discovery**: Find all NixOS options used/defined
- **Package relationships**: Understand overlay modifications
- **Security scanning**: Detect insecure patterns
- **Performance analysis**: Find evaluation bottlenecks
- **Dead code detection**: Find unused definitions

### 3. Workflow Integration
- **Convert Nix to CIM workflows**: Generate graph workflows from Nix
- **Workflow templates**: Create Nix from workflow patterns
- **CI/CD integration**: Analyze and generate GitHub Actions
- **Build orchestration**: Understand build dependencies

## Implementation Architecture

### Phase 1: Core Parser Infrastructure

```rust
// cim-domain-nix/src/parser/mod.rs
pub mod ast;
pub mod analyzer;
pub mod manipulator;
pub mod validator;

use rnix::{SyntaxNode, SyntaxKind};
use rowan::{GreenNode, TextRange};

/// Core parser that preserves all formatting
pub struct NixParser {
    /// Parser configuration
    config: ParserConfig,
}

pub struct ParserConfig {
    /// Preserve comments
    pub preserve_comments: bool,
    /// Validate while parsing
    pub validate: bool,
    /// Parse included files recursively
    pub follow_imports: bool,
    /// Maximum recursion depth
    pub max_depth: usize,
}

impl NixParser {
    pub fn parse_file(&self, path: &Path) -> Result<ParsedFile> {
        let content = std::fs::read_to_string(path)?;
        self.parse_string(&content, Some(path))
    }

    pub fn parse_string(&self, content: &str, source: Option<&Path>) -> Result<ParsedFile> {
        let parsed = rnix::parse(content);
        
        // Collect errors
        let errors: Vec<_> = parsed.errors().iter()
            .map(|e| ParseError::from_rnix(e, source))
            .collect();

        // Validate if requested
        if self.config.validate && errors.is_empty() {
            let validation_errors = self.validate_ast(&parsed.node())?;
            errors.extend(validation_errors);
        }

        Ok(ParsedFile {
            ast: parsed.node(),
            green: parsed.green(),
            source: source.map(|p| p.to_path_buf()),
            errors,
            content: content.to_string(),
        })
    }
}

pub struct ParsedFile {
    /// The syntax tree
    pub ast: SyntaxNode,
    /// The green tree (for incremental updates)
    pub green: GreenNode,
    /// Source file path
    pub source: Option<PathBuf>,
    /// Parse errors
    pub errors: Vec<ParseError>,
    /// Original content
    pub content: String,
}
```

### Phase 2: Specialized File Type Support

```rust
// cim-domain-nix/src/parser/flake.rs
use super::*;
use crate::value_objects::{FlakeInputs, FlakeOutputs};

pub struct FlakeFile {
    parsed: ParsedFile,
    metadata: FlakeMetadata,
}

pub struct FlakeMetadata {
    pub description: Option<String>,
    pub inputs: FlakeInputs,
    pub outputs: FlakeOutputs,
    pub nix_config: Option<HashMap<String, String>>,
}

impl FlakeFile {
    pub fn parse(path: &Path) -> Result<Self> {
        let parser = NixParser::default();
        let parsed = parser.parse_file(path)?;
        let metadata = Self::extract_metadata(&parsed)?;
        
        Ok(Self { parsed, metadata })
    }

    pub fn add_input(&mut self, name: &str, url: &str) -> Result<()> {
        // Use AST manipulation to add input
        let inputs_node = self.find_or_create_inputs_node()?;
        self.add_attribute_to_set(inputs_node, name, url)?;
        self.regenerate_content()?;
        Ok(())
    }

    pub fn analyze_dependencies(&self) -> DependencyGraph {
        // Build complete dependency graph
        DependencyAnalyzer::analyze(&self.metadata.inputs)
    }

    pub fn to_workflow(&self) -> Result<WorkflowGraph> {
        // Convert flake to CIM workflow
        FlakeToWorkflowConverter::convert(self)
    }
}

// cim-domain-nix/src/parser/nixos_module.rs
pub struct NixOSModule {
    parsed: ParsedFile,
    module_info: ModuleInfo,
}

pub struct ModuleInfo {
    pub imports: Vec<PathBuf>,
    pub options: HashMap<String, OptionDefinition>,
    pub config: HashMap<String, Value>,
    pub meta: ModuleMeta,
}

impl NixOSModule {
    pub fn analyze_options(&self) -> OptionAnalysis {
        // Deep analysis of all options
        OptionAnalyzer::analyze(&self.module_info)
    }

    pub fn find_dependencies(&self) -> Vec<ModuleDependency> {
        // Find all module dependencies
        DependencyFinder::find_in_module(&self.module_info)
    }
}
```

### Phase 3: Analysis Engine

```rust
// cim-domain-nix/src/analyzer/mod.rs
pub mod dependency;
pub mod security;
pub mod performance;
pub mod dead_code;

use petgraph::graph::{DiGraph, NodeIndex};

pub struct NixAnalyzer {
    /// Parsed files cache
    cache: HashMap<PathBuf, ParsedFile>,
    /// Dependency graph
    dep_graph: DiGraph<FileNode, DependencyEdge>,
}

impl NixAnalyzer {
    pub async fn analyze_repository(&mut self, repo_path: &Path) -> Result<AnalysisReport> {
        // Find all Nix files
        let nix_files = self.find_nix_files(repo_path).await?;
        
        // Parse all files in parallel
        let parsed_files = self.parse_files_parallel(nix_files).await?;
        
        // Build dependency graph
        self.build_dependency_graph(&parsed_files)?;
        
        // Run analyzers
        let security = SecurityAnalyzer::analyze(&parsed_files)?;
        let performance = PerformanceAnalyzer::analyze(&parsed_files)?;
        let dead_code = DeadCodeAnalyzer::analyze(&parsed_files, &self.dep_graph)?;
        
        Ok(AnalysisReport {
            files_analyzed: parsed_files.len(),
            dependency_graph: self.dep_graph.clone(),
            security_issues: security,
            performance_issues: performance,
            dead_code: dead_code,
        })
    }

    pub fn generate_workflow(&self, entry_point: &Path) -> Result<WorkflowGraph> {
        // Generate CIM workflow from Nix configuration
        let analyzer = WorkflowGenerator::new(&self.cache, &self.dep_graph);
        analyzer.generate_from_entry(entry_point)
    }
}

// cim-domain-nix/src/analyzer/security.rs
pub struct SecurityAnalyzer;

impl SecurityAnalyzer {
    pub fn analyze(files: &[ParsedFile]) -> Result<Vec<SecurityIssue>> {
        let mut issues = Vec::new();
        
        for file in files {
            // Check for insecure patterns
            issues.extend(Self::check_insecure_fetchers(&file.ast)?);
            issues.extend(Self::check_impure_functions(&file.ast)?);
            issues.extend(Self::check_unsafe_derivations(&file.ast)?);
            issues.extend(Self::check_fixed_output_hashes(&file.ast)?);
        }
        
        Ok(issues)
    }
}
```

### Phase 4: Manipulation Engine

```rust
// cim-domain-nix/src/manipulator/mod.rs
use rnix::SyntaxNode;
use rowan::{GreenNodeBuilder, NodeOrToken};

pub struct NixManipulator {
    /// Original AST
    original: SyntaxNode,
    /// Builder for new tree
    builder: GreenNodeBuilder,
}

impl NixManipulator {
    pub fn new(ast: SyntaxNode) -> Self {
        Self {
            original: ast,
            builder: GreenNodeBuilder::new(),
        }
    }

    pub fn add_flake_input(&mut self, name: &str, url: &str) -> Result<()> {
        // Sophisticated AST manipulation
        self.visit_and_modify(|node| {
            if Self::is_inputs_attrset(node) {
                self.insert_attribute(node, name, url)
            } else {
                Ok(())
            }
        })
    }

    pub fn refactor_module_imports(&mut self, changes: &[ImportChange]) -> Result<()> {
        // Refactor imports across module
        for change in changes {
            self.apply_import_change(change)?;
        }
        Ok(())
    }

    pub fn optimize_derivation(&mut self, drv_name: &str) -> Result<OptimizationReport> {
        // Apply performance optimizations
        let optimizer = DerivationOptimizer::new();
        optimizer.optimize(&mut self.builder, drv_name)
    }

    pub fn generate(&self) -> String {
        // Generate formatted Nix code
        let formatter = NixFormatter::new();
        formatter.format(self.builder.finish())
    }
}
```

### Phase 5: Git Integration

**Status: ✅ COMPLETED**

The Git integration has been successfully implemented with the following features:

### Features Implemented

1. **Flake.lock History Tracking**
   - Analyze flake.lock changes over time
   - Track dependency updates per input
   - Identify update patterns and frequencies

2. **Dependency Change Analysis**
   - Compare dependencies between commits
   - Identify added, removed, and updated inputs
   - Track version changes with commit hashes

3. **Nix File Change Tracking**
   - Monitor all Nix file changes in Git history
   - Categorize changes by type (added, modified, deleted, renamed)
   - Associate changes with commits and authors

4. **Update Pattern Analysis**
   - Calculate average update frequency
   - Identify most active update times (day/hour)
   - Detect batch updates (multiple inputs updated together)
   - Generate recommendations for update practices

5. **Stale Dependency Detection**
   - Identify inputs that haven't been updated recently
   - Configurable staleness threshold
   - Prioritized recommendations for updates

### Key Components

- **GitNixAnalyzer**: Core analyzer for Git-Nix integration
- **FlakeLockTracker**: Specialized tracking for flake.lock changes
- **GitFlakeService**: Service for managing Git-backed flakes
- **Git URL to Flake Reference Mapping**: Support for GitHub, GitLab, Sourcehut

### Usage Example

```bash
# Analyze flake.lock history in current directory
cargo run --example git_integration_demo

# Analyze specific repository
cargo run --example git_integration_demo /path/to/repo

# Compare dependency changes between commits
cargo run --example git_integration_demo . --compare abc123 def456
```

### Phase 6: Workflow Generation

```rust
// cim-domain-nix/src/workflow/generator.rs
use crate::domain::graph::{GraphNode, GraphEdge};

pub struct WorkflowGenerator {
    analyzer: NixAnalyzer,
}

impl WorkflowGenerator {
    pub fn from_flake(&self, flake: &FlakeFile) -> Result<WorkflowGraph> {
        let mut graph = WorkflowGraph::new();
        
        // Create nodes for each output
        for (name, output) in &flake.metadata.outputs.packages {
            let node = self.create_package_node(name, output)?;
            graph.add_node(node);
        }
        
        // Add dependencies as edges
        for (from, to) in self.analyze_build_dependencies(flake)? {
            graph.add_edge(from, to, EdgeType::BuildDependency);
        }
        
        // Add workflow metadata
        graph.set_metadata(WorkflowMetadata {
            source: WorkflowSource::NixFlake(flake.source.clone()),
            created_from: flake.metadata.description.clone(),
        });
        
        Ok(graph)
    }

    pub fn from_nixos_config(&self, config: &NixOSModule) -> Result<WorkflowGraph> {
        // Generate workflow from NixOS configuration
        let mut graph = WorkflowGraph::new();
        
        // Create nodes for each service
        for service in self.extract_services(config)? {
            let node = self.create_service_node(service)?;
            graph.add_node(node);
        }
        
        // Add service dependencies
        self.add_service_dependencies(&mut graph, config)?;
        
        Ok(graph)
    }
}
```

## Integration with CIM

### Event Integration

```rust
// cim-domain-nix/src/events/analysis.rs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NixAnalysisEvent {
    RepositoryAnalyzed {
        repo_path: PathBuf,
        analysis_id: Uuid,
        files_count: usize,
        issues_found: usize,
        duration: Duration,
    },
    SecurityIssueDetected {
        file: PathBuf,
        issue: SecurityIssue,
        severity: Severity,
    },
    WorkflowGenerated {
        source: WorkflowSource,
        workflow_id: Uuid,
        nodes_count: usize,
        edges_count: usize,
    },
    DependencyGraphUpdated {
        graph_id: Uuid,
        nodes_added: Vec<String>,
        edges_added: Vec<(String, String)>,
    },
}
```

### Query Support

```rust
// cim-domain-nix/src/queries/mod.rs
pub struct NixQueryHandler;

impl NixQueryHandler {
    pub async fn find_option_usage(&self, option: &str) -> Result<Vec<OptionUsage>> {
        // Find all uses of a NixOS option
    }

    pub async fn get_package_closure(&self, package: &str) -> Result<PackageClosure> {
        // Get full closure of a package
    }

    pub async fn analyze_flake_inputs(&self, flake_path: &Path) -> Result<InputAnalysis> {
        // Deep analysis of flake inputs
    }

    pub async fn suggest_optimizations(&self, file: &Path) -> Result<Vec<Optimization>> {
        // Suggest performance optimizations
    }
}
```

## Performance Considerations

1. **Parallel Parsing**: Use rayon for parallel file parsing
2. **Incremental Updates**: Use rowan's green tree for incremental parsing
3. **Caching**: Cache parsed files and analysis results
4. **Lazy Loading**: Only parse imports when needed
5. **Memory Mapping**: Use mmap for large files like nixpkgs

## Testing Strategy

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_flake_manipulation() {
        let flake = r#"
        {
          description = "Test flake";
          inputs.nixpkgs.url = "github:NixOS/nixpkgs";
          outputs = { self, nixpkgs }: {
            packages.x86_64-linux.hello = nixpkgs.legacyPackages.x86_64-linux.hello;
          };
        }
        "#;

        let mut parsed = FlakeFile::parse_string(flake).unwrap();
        parsed.add_input("flake-utils", "github:numtide/flake-utils").unwrap();
        
        assert!(parsed.to_string().contains("flake-utils"));
        assert_eq!(parsed.metadata.inputs.inputs.len(), 2);
    }

    #[test]
    fn test_security_analysis() {
        let unsafe_nix = r#"
        {
          # Unsafe: using builtins.fetchurl without hash
          src = builtins.fetchurl {
            url = "https://example.com/source.tar.gz";
          };
        }
        "#;

        let issues = SecurityAnalyzer::analyze_string(unsafe_nix).unwrap();
        assert!(!issues.is_empty());
        assert_eq!(issues[0].severity, Severity::High);
    }

    #[test]
    fn test_workflow_generation() {
        let flake = load_test_flake("complex_flake.nix");
        let workflow = WorkflowGenerator::from_flake(&flake).unwrap();
        
        assert!(workflow.nodes().len() > 0);
        assert!(workflow.validate().is_ok());
    }
}
```

## Deliverables

1. **Complete AST manipulation** for all Nix file types
2. **Repository-wide analysis** with performance metrics
3. **Security scanning** with configurable rules
4. **Workflow generation** from Nix configurations
5. **Git integration** for historical analysis
6. **Rich query API** for complex analysis
7. **Performance optimizations** for large codebases
8. **Comprehensive test suite** with real-world examples

## Success Metrics

- Parse entire nixpkgs in < 30 seconds
- Generate accurate dependency graphs for complex flakes
- Detect 95%+ of common security issues
- Convert NixOS configurations to CIM workflows
- Handle incremental updates in < 100ms
- Support all major Nix file types and patterns 

## Advanced Features

### Phase 7: Home Manager Integration ✅ COMPLETED

```rust
// cim-domain-nix/src/home_manager/mod.rs
use crate::parser::{NixParser, ParsedFile};
use crate::value_objects::{HomeConfiguration, ProgramConfig};

pub struct HomeManagerAnalyzer {
    parser: NixParser,
    config_cache: HashMap<String, HomeConfiguration>,
}

impl HomeManagerAnalyzer {
    pub async fn analyze_home_config(&mut self, path: &Path) -> Result<HomeAnalysis> {
        let parsed = self.parser.parse_file(path)?;
        let config = self.extract_home_config(&parsed)?;
        
        let analysis = HomeAnalysis {
            programs: self.analyze_programs(&config)?,
            services: self.analyze_services(&config)?,
            dotfiles: self.find_dotfile_mappings(&config)?,
            conflicts: self.detect_conflicts(&config)?,
            suggestions: self.generate_suggestions(&config)?,
        };
        
        Ok(analysis)
    }

    pub fn analyze_programs(&self, config: &HomeConfiguration) -> Result<Vec<ProgramAnalysis>> {
        let mut analyses = Vec::new();
        
        for (name, program_config) in &config.programs {
            let analysis = ProgramAnalysis {
                name: name.clone(),
                enabled: program_config.is_enabled(),
                dependencies: self.find_program_dependencies(program_config)?,
                configuration_complexity: self.calculate_complexity(program_config),
                security_score: self.assess_security(program_config)?,
            };
            analyses.push(analysis);
        }
        
        Ok(analyses)
    }

    pub fn migrate_from_dotfiles(&self, dotfiles_dir: &Path) -> Result<HomeConfiguration> {
        // Analyze existing dotfiles and generate Home Manager config
        let dotfiles = self.scan_dotfiles(dotfiles_dir)?;
        let mut config = HomeConfiguration::new();
        
        for dotfile in dotfiles {
            match self.identify_program(&dotfile) {
                Some(program) => {
                    let program_config = self.convert_to_home_manager(&dotfile, &program)?;
                    config.add_program(program, program_config);
                }
                None => {
                    // Add as raw file mapping
                    config.add_file_mapping(dotfile.source, dotfile.target);
                }
            }
        }
        
        Ok(config)
    }
}

// cim-domain-nix/src/home_manager/program_converter.rs
pub struct ProgramConverter {
    converters: HashMap<String, Box<dyn DotfileConverter>>,
}

impl ProgramConverter {
    pub fn new() -> Self {
        let mut converters = HashMap::new();
        
        // Register converters for common programs
        converters.insert("vim".to_string(), Box::new(VimConverter::new()));
        converters.insert("git".to_string(), Box::new(GitConverter::new()));
        converters.insert("zsh".to_string(), Box::new(ZshConverter::new()));
        converters.insert("tmux".to_string(), Box::new(TmuxConverter::new()));
        
        Self { converters }
    }

    pub fn convert(&self, program: &str, dotfile: &Path) -> Result<ProgramConfig> {
        self.converters
            .get(program)
            .ok_or(Error::UnsupportedProgram(program.to_string()))?
            .convert(dotfile)
    }
}
```

### Phase 8: Nix Template System

```rust
// cim-domain-nix/src/templates/mod.rs
use handlebars::Handlebars;
use serde_json::Value as JsonValue;

pub struct NixTemplateEngine {
    registry: Handlebars<'static>,
    templates: HashMap<String, Template>,
}

#[derive(Debug, Clone)]
pub struct Template {
    pub name: String,
    pub description: String,
    pub category: TemplateCategory,
    pub parameters: Vec<TemplateParameter>,
    pub content: String,
}

#[derive(Debug, Clone)]
pub enum TemplateCategory {
    Flake,
    Module,
    Package,
    Service,
    Development,
    CI,
}

impl NixTemplateEngine {
    pub fn new() -> Result<Self> {
        let mut engine = Self {
            registry: Handlebars::new(),
            templates: HashMap::new(),
        };
        
        // Load built-in templates
        engine.load_builtin_templates()?;
        
        // Register custom helpers
        engine.register_helpers()?;
        
        Ok(engine)
    }

    pub fn generate(&self, template_name: &str, params: &JsonValue) -> Result<String> {
        let template = self.templates
            .get(template_name)
            .ok_or(Error::TemplateNotFound(template_name.to_string()))?;
        
        // Validate parameters
        self.validate_parameters(template, params)?;
        
        // Render template
        let rendered = self.registry.render(template_name, params)?;
        
        // Format the output
        let formatted = NixFormatter::format_string(&rendered)?;
        
        Ok(formatted)
    }

    pub fn create_flake_from_workflow(&self, workflow: &WorkflowGraph) -> Result<String> {
        let params = self.workflow_to_template_params(workflow)?;
        self.generate("workflow-flake", &params)
    }

    fn load_builtin_templates(&mut self) -> Result<()> {
        // Rust project flake
        self.register_template(Template {
            name: "rust-flake".to_string(),
            description: "Rust project with Nix flake".to_string(),
            category: TemplateCategory::Development,
            parameters: vec![
                TemplateParameter::new("project_name", "Project name", true),
                TemplateParameter::new("rust_version", "Rust version", false)
                    .with_default("stable"),
            ],
            content: include_str!("templates/rust-flake.nix.hbs"),
        })?;

        // NixOS module
        self.register_template(Template {
            name: "nixos-module".to_string(),
            description: "NixOS module template".to_string(),
            category: TemplateCategory::Module,
            parameters: vec![
                TemplateParameter::new("module_name", "Module name", true),
                TemplateParameter::new("enable_option", "Include enable option", false)
                    .with_default("true"),
            ],
            content: include_str!("templates/nixos-module.nix.hbs"),
        })?;

        // GitHub Actions workflow
        self.register_template(Template {
            name: "github-actions-nix".to_string(),
            description: "GitHub Actions with Nix".to_string(),
            category: TemplateCategory::CI,
            parameters: vec![
                TemplateParameter::new("workflow_name", "Workflow name", true),
                TemplateParameter::new("cachix_name", "Cachix cache name", false),
            ],
            content: include_str!("templates/github-actions.yml.hbs"),
        })?;

        Ok(())
    }
}

// cim-domain-nix/src/templates/generator.rs
pub struct ProjectGenerator {
    template_engine: NixTemplateEngine,
}

impl ProjectGenerator {
    pub async fn scaffold_project(&self, config: ProjectConfig) -> Result<GeneratedProject> {
        let mut project = GeneratedProject::new(&config.name);
        
        // Generate flake.nix
        let flake = self.template_engine.generate("project-flake", &config.to_params())?;
        project.add_file("flake.nix", flake);
        
        // Generate shell.nix for compatibility
        if config.include_shell_nix {
            let shell = self.template_engine.generate("shell-nix", &config.to_params())?;
            project.add_file("shell.nix", shell);
        }
        
        // Generate .envrc for direnv
        if config.use_direnv {
            project.add_file(".envrc", "use flake\n");
        }
        
        // Generate GitHub Actions if requested
        if let Some(ci_config) = &config.ci_config {
            let workflow = self.generate_ci_workflow(ci_config)?;
            project.add_file(".github/workflows/ci.yml", workflow);
        }
        
        Ok(project)
    }
}
```

### Phase 9: Real-time Analysis and LSP Support

```rust
// cim-domain-nix/src/lsp/mod.rs
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};

pub struct NixLanguageServer {
    client: Client,
    analyzer: Arc<Mutex<NixAnalyzer>>,
    document_cache: Arc<RwLock<HashMap<Url, DocumentState>>>,
}

#[tower_lsp::async_trait]
impl LanguageServer for NixLanguageServer {
    async fn initialize(&self, _: InitializeParams) -> tower_lsp::Result<InitializeResult> {
        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Options(
                    TextDocumentSyncOptions {
                        open_close: Some(true),
                        change: Some(TextDocumentSyncKind::INCREMENTAL),
                        ..Default::default()
                    },
                )),
                completion_provider: Some(CompletionOptions {
                    resolve_provider: Some(true),
                    trigger_characters: Some(vec![".".to_string(), " ".to_string()]),
                    ..Default::default()
                }),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                definition_provider: Some(OneOf::Left(true)),
                references_provider: Some(OneOf::Left(true)),
                document_symbol_provider: Some(OneOf::Left(true)),
                workspace_symbol_provider: Some(OneOf::Left(true)),
                code_action_provider: Some(CodeActionProviderCapability::Simple(true)),
                code_lens_provider: Some(CodeLensOptions {
                    resolve_provider: Some(true),
                }),
                semantic_tokens_provider: Some(
                    SemanticTokensServerCapabilities::SemanticTokensOptions(
                        SemanticTokensOptions {
                            legend: SemanticTokensLegend {
                                token_types: vec![
                                    SemanticTokenType::KEYWORD,
                                    SemanticTokenType::VARIABLE,
                                    SemanticTokenType::FUNCTION,
                                    SemanticTokenType::STRING,
                                    SemanticTokenType::NUMBER,
                                    SemanticTokenType::COMMENT,
                                ],
                                token_modifiers: vec![],
                            },
                            ..Default::default()
                        },
                    ),
                ),
                ..Default::default()
            },
            ..Default::default()
        })
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let uri = params.text_document.uri;
        let content = params.text_document.text;
        
        // Parse and analyze
        let diagnostics = self.analyze_document(&uri, &content).await;
        
        // Send diagnostics
        self.client
            .publish_diagnostics(uri, diagnostics, None)
            .await;
    }

    async fn completion(
        &self,
        params: CompletionParams,
    ) -> tower_lsp::Result<Option<CompletionResponse>> {
        let uri = params.text_document_position.text_document.uri;
        let position = params.text_document_position.position;
        
        let completions = self.get_completions(&uri, position).await?;
        
        Ok(Some(CompletionResponse::Array(completions)))
    }

    async fn hover(&self, params: HoverParams) -> tower_lsp::Result<Option<Hover>> {
        let uri = params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;
        
        let hover_info = self.get_hover_info(&uri, position).await?;
        
        Ok(hover_info.map(|info| Hover {
            contents: HoverContents::Markup(MarkupContent {
                kind: MarkupKind::Markdown,
                value: info,
            }),
            range: None,
        }))
    }

    async fn code_action(&self, params: CodeActionParams) -> tower_lsp::Result<Option<CodeActionResponse>> {
        let uri = params.text_document.uri;
        let range = params.range;
        
        let mut actions = Vec::new();
        
        // Quick fixes for diagnostics
        for diagnostic in &params.context.diagnostics {
            if let Some(fixes) = self.get_quick_fixes(&uri, diagnostic).await? {
                actions.extend(fixes);
            }
        }
        
        // Refactoring actions
        let refactorings = self.get_refactoring_actions(&uri, range).await?;
        actions.extend(refactorings);
        
        Ok(Some(actions))
    }
}

impl NixLanguageServer {
    async fn analyze_document(&self, uri: &Url, content: &str) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        
        // Parse the document
        let parsed = match self.analyzer.lock().await.parse_string(content) {
            Ok(parsed) => parsed,
            Err(e) => {
                diagnostics.push(Diagnostic {
                    range: Range::default(),
                    severity: Some(DiagnosticSeverity::ERROR),
                    message: format!("Parse error: {}", e),
                    ..Default::default()
                });
                return diagnostics;
            }
        };
        
        // Syntax errors
        for error in &parsed.errors {
            diagnostics.push(self.error_to_diagnostic(error));
        }
        
        // Security issues
        if let Ok(security_issues) = SecurityAnalyzer::analyze(&[parsed.clone()]) {
            for issue in security_issues {
                diagnostics.push(self.security_issue_to_diagnostic(&issue));
            }
        }
        
        // Best practice violations
        let violations = BestPracticeAnalyzer::analyze(&parsed);
        for violation in violations {
            diagnostics.push(self.violation_to_diagnostic(&violation));
        }
        
        diagnostics
    }

    async fn get_completions(&self, uri: &Url, position: Position) -> Result<Vec<CompletionItem>> {
        let document = self.document_cache.read().await.get(uri).cloned();
        let Some(doc) = document else {
            return Ok(Vec::new());
        };
        
        let context = self.get_completion_context(&doc, position)?;
        
        match context {
            CompletionContext::FlakeInput => self.complete_flake_inputs(),
            CompletionContext::NixpkgsPackage => self.complete_nixpkgs_packages().await,
            CompletionContext::NixOSOption => self.complete_nixos_options().await,
            CompletionContext::Attribute(parent) => self.complete_attributes(&parent).await,
            _ => Ok(Vec::new()),
        }
    }
}
```

### Phase 10: Cross-Domain Integration

```rust
// cim-domain-nix/src/integration/mod.rs
pub mod graph_integration;
pub mod workflow_integration;
pub mod git_integration;

use cim_domain_graph::{GraphCommand, GraphNode};
use cim_domain_workflow::{WorkflowCommand, WorkflowStep};

pub struct NixToCIMBridge {
    nix_analyzer: NixAnalyzer,
    graph_client: GraphDomainClient,
    workflow_client: WorkflowDomainClient,
}

impl NixToCIMBridge {
    pub async fn import_flake_as_workflow(
        &self,
        flake_path: &Path,
    ) -> Result<WorkflowId> {
        // Parse and analyze flake
        let flake = FlakeFile::parse(flake_path)?;
        let deps = flake.analyze_dependencies();
        
        // Create workflow graph
        let mut workflow_commands = Vec::new();
        
        // Create workflow
        let workflow_id = WorkflowId::new();
        workflow_commands.push(WorkflowCommand::CreateWorkflow {
            id: workflow_id,
            name: format!("Nix Build: {}", flake.metadata.description.as_deref().unwrap_or("Unnamed")),
            description: Some("Generated from Nix flake".to_string()),
        });
        
        // Add steps for each package
        for (name, package) in &flake.metadata.outputs.packages {
            let step_id = WorkflowStepId::new();
            workflow_commands.push(WorkflowCommand::AddStep {
                workflow_id,
                step: WorkflowStep {
                    id: step_id,
                    name: format!("Build {}", name),
                    action: WorkflowAction::RunCommand {
                        command: format!("nix build .#{}", name),
                    },
                    inputs: self.get_step_inputs(&deps, name)?,
                    outputs: vec![WorkflowOutput::Artifact {
                        name: name.clone(),
                        path: format!("result-{}", name),
                    }],
                },
            });
        }
        
        // Execute commands
        for cmd in workflow_commands {
            self.workflow_client.execute(cmd).await?;
        }
        
        Ok(workflow_id)
    }

    pub async fn visualize_nix_dependencies(
        &self,
        entry_point: &Path,
    ) -> Result<GraphId> {
        // Analyze dependencies
        let analysis = self.nix_analyzer.analyze_repository(entry_point.parent().unwrap()).await?;
        
        // Create graph
        let graph_id = GraphId::new();
        let mut graph_commands = Vec::new();
        
        graph_commands.push(GraphCommand::CreateGraph {
            id: graph_id,
            name: "Nix Dependencies".to_string(),
        });
        
        // Add nodes for each file
        let mut node_map = HashMap::new();
        for (idx, node) in analysis.dependency_graph.node_indices().enumerate() {
            let file_node = &analysis.dependency_graph[node];
            let node_id = NodeId::new();
            
            node_map.insert(node, node_id);
            
            graph_commands.push(GraphCommand::AddNode {
                graph_id,
                node: GraphNode {
                    id: node_id,
                    label: file_node.path.display().to_string(),
                    node_type: NodeType::Custom("nix-file".to_string()),
                    metadata: self.create_node_metadata(&file_node),
                },
            });
        }
        
        // Add edges for dependencies
        for edge in analysis.dependency_graph.edge_indices() {
            let (source, target) = analysis.dependency_graph.edge_endpoints(edge).unwrap();
            
            graph_commands.push(GraphCommand::AddEdge {
                graph_id,
                source: node_map[&source],
                target: node_map[&target],
                edge_type: EdgeType::Dependency,
                metadata: HashMap::new(),
            });
        }
        
        // Execute commands
        for cmd in graph_commands {
            self.graph_client.execute(cmd).await?;
        }
        
        Ok(graph_id)
    }
}
```

## Phase 10: Formatter Integration

**Status: ✅ COMPLETED**

The formatter integration has been successfully implemented with the following features:

### Supported Formatters
- **nixpkgs-fmt** - The official formatter
- **alejandra** - Opinionated formatter
- **nixfmt** - Classic formatter
- **nixfmt-rfc** - RFC-style formatting

### Features Implemented
1. **Auto-detection** - Automatically detects formatter based on project configuration
2. **Check mode** - Can check formatting without modifying files
3. **Format mode** - Can format files in place
4. **Directory support** - Can format entire directories recursively
5. **Integration with analyzer** - Formatting checks integrated into the main analyzer

### Usage Examples

```bash
# Check formatting in analyzer
cargo run --example analyzer_demo path/to/project --check-formatting

# Format a single file
cargo run --example formatter_demo path/to/file.nix

# Check formatting without modifying
cargo run --example formatter_demo path/to/project --check

# Use specific formatter
cargo run --example formatter_demo path/to/project --formatter=alejandra
```

### API Usage

```rust
use cim_domain_nix::formatter::{NixFormatter, FormatterService};

// Auto-detect formatter
let formatter = NixFormatter::detect_from_project(path).await?;

// Create service
let service = FormatterService::new(formatter);

// Format files
let report = service.format_directory(path).await?;
println!("{}", report.summary());
```

## Phase 11: Future Enhancements

## Conclusion

The enhanced Nix domain will provide unprecedented capabilities for working with Nix files programmatically. By combining deep AST manipulation, comprehensive analysis, and seamless CIM integration, this domain will enable powerful workflows and tooling that were previously impossible.

The implementation focuses on correctness, performance, and usability, ensuring that the domain can handle everything from simple flake edits to analyzing the entire nixpkgs repository. With proper abstraction and event-driven architecture, the Nix domain will be a cornerstone of the CIM ecosystem for Nix users. 
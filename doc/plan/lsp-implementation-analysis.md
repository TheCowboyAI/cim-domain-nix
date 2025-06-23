# LSP Implementation Analysis for Nix Domain

## Current Assets That Support LSP

### 1. **Parsing & Syntax Analysis** ✅
- We have `NixFile::parse_string()` that produces AST and errors
- Parse errors already include position information
- Can easily map to LSP `Diagnostic` messages

### 2. **Semantic Analysis** ✅
- Security analyzer detects issues with suggestions
- Performance analyzer identifies problematic patterns
- Dead code analyzer finds unused definitions
- All include file paths and line numbers

### 3. **Formatting** ✅
- Multiple formatter support already implemented
- Can provide format-on-save and format-document actions

### 4. **Dependency Analysis** ✅
- Dependency graph can power "Go to Definition" and "Find References"
- Import resolution already implemented

## LSP Features to Implement

### Phase 1: Core Features (1-2 weeks)
1. **Text Document Synchronization**
   - Track open documents
   - Handle incremental updates
   - Relatively simple with our existing parser

2. **Diagnostics**
   - Convert our analyzer results to LSP diagnostics
   - Real-time analysis on document change
   - Mostly mapping existing data structures

3. **Formatting**
   - Wire up existing formatter to LSP format requests
   - Already have the logic, just need protocol handling

### Phase 2: Navigation (1-2 weeks)
1. **Go to Definition**
   - Use dependency graph to resolve imports
   - Navigate to attribute definitions
   - Leverage existing AST traversal

2. **Find References**
   - Search for usage of variables/functions
   - Use our existing AST analysis

3. **Document Symbols**
   - Extract top-level definitions
   - Create outline view
   - Simple AST traversal

### Phase 3: Intelligence Features (2-3 weeks)
1. **Hover Information**
   - Show type information
   - Display documentation
   - Show evaluation results for simple expressions

2. **Code Completion**
   - Complete attribute names
   - Suggest nixpkgs packages
   - Context-aware completions

3. **Code Actions**
   - Quick fixes for security issues
   - Auto-add missing imports
   - Convert between attribute set styles

### Phase 4: Advanced Features (2-3 weeks)
1. **Semantic Highlighting**
   - Distinguish between different token types
   - Highlight based on semantic meaning

2. **Code Lens**
   - Show reference counts
   - Display complexity metrics
   - Inline security warnings

3. **Rename Support**
   - Rename variables across scope
   - Update references

## Implementation Strategy

### 1. **Use tower-lsp**
```toml
[dependencies]
tower-lsp = "0.20"
tokio = { version = "1.40", features = ["full"] }
```

### 2. **Architecture**
```rust
pub struct NixLanguageServer {
    // Reuse existing components
    parser: NixParser,
    analyzer: NixAnalyzer,
    formatter: FormatterService,
    
    // LSP-specific state
    document_cache: HashMap<Url, DocumentState>,
    client: Client,
}
```

### 3. **Minimal Working Example**
```rust
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};

#[tower_lsp::async_trait]
impl LanguageServer for NixLanguageServer {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::INCREMENTAL
                )),
                diagnostic_provider: Some(DiagnosticServerCapabilities::Options(
                    DiagnosticOptions::default()
                )),
                document_formatting_provider: Some(OneOf::Left(true)),
                ..Default::default()
            },
            ..Default::default()
        })
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        // Parse document
        let content = self.get_document_content(&params.text_document.uri);
        let parsed = self.parser.parse_string(&content, None);
        
        // Run analyzers
        let diagnostics = self.convert_to_lsp_diagnostics(&parsed).await;
        
        // Publish diagnostics
        self.client
            .publish_diagnostics(params.text_document.uri, diagnostics, None)
            .await;
    }
}
```

## Estimated Effort

### Minimal Viable LSP (1 week)
- Text synchronization
- Diagnostics from existing analyzers
- Document formatting

### Full-Featured LSP (4-6 weeks)
- All navigation features
- Intelligent completions
- Advanced code actions

### Integration with Editors
- VS Code extension: 1 week
- Neovim config: Simple with built-in LSP client
- Emacs/Others: Community can contribute

## Benefits of Adding LSP

1. **Immediate Value**: Real-time feedback on security/performance issues
2. **Developer Experience**: IDE features for Nix development
3. **Adoption**: Makes the analyzer accessible to all editor users
4. **Ecosystem**: Positions CIM as a key Nix development tool

## Challenges

1. **Completion Data**: Need to index nixpkgs for package completions
2. **Performance**: Real-time analysis needs to be fast
3. **Nix Evaluation**: Some features may require evaluating Nix expressions
4. **Testing**: LSP testing can be complex

## Recommendation

LSP support is highly feasible and would provide significant value. The existing analyzer infrastructure provides 60-70% of what's needed. I recommend:

1. Start with a minimal LSP that exposes existing analysis
2. Incrementally add features based on user feedback
3. Consider making this a separate crate (`cim-nix-lsp`) to keep concerns separated
4. Publish as a standalone binary that any editor can use

The combination of our existing analysis capabilities with LSP would make this one of the most advanced Nix development tools available. 
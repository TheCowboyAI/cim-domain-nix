//! Advanced Nix parser with full AST manipulation capabilities

use rnix::{SyntaxKind, SyntaxNode};
use std::path::{Path, PathBuf};

use super::ast::{
    AttrPath, AttrPathSegment, BinaryOperator, Binding, BindingValue, FunctionParam, LocatedAst,
    Location, NixAst, PatternField, UnaryOperator,
};
use crate::{NixDomainError, Result};

/// Advanced parser with full AST capabilities
pub struct AdvancedParser {
    /// Current file being parsed
    current_file: Option<PathBuf>,
}

impl Default for AdvancedParser {
    fn default() -> Self {
        Self::new()
    }
}

impl AdvancedParser {
    /// Create a new advanced parser
    pub fn new() -> Self {
        Self { current_file: None }
    }

    /// Parse a file into our AST representation
    pub fn parse_file(&self, path: &Path) -> Result<LocatedAst> {
        let content = std::fs::read_to_string(path).map_err(NixDomainError::IoError)?;

        self.parse_string(&content)
    }

    /// Parse a string into our AST representation
    pub fn parse_string(&self, content: &str) -> Result<LocatedAst> {
        // Parse with rnix
        let parsed = rnix::Root::parse(content);

        if !parsed.errors().is_empty() {
            let errors: Vec<String> = parsed
                .errors()
                .iter()
                .map(std::string::ToString::to_string)
                .collect();
            return Err(NixDomainError::ParseError(errors.join("; ")));
        }

        let root = parsed.syntax();
        let ast = self.convert_node(&root)?;

        Ok(LocatedAst {
            ast,
            location: self.node_location(&root),
        })
    }

    /// Convert a syntax node to our AST representation
    fn convert_node(&self, node: &SyntaxNode) -> Result<NixAst> {
        match node.kind() {
            // Literals
            SyntaxKind::NODE_LITERAL => self.convert_literal(node),
            SyntaxKind::NODE_STRING => self.convert_string(node),

            // Identifiers
            SyntaxKind::NODE_IDENT => self.convert_identifier(node),

            // Collections
            SyntaxKind::NODE_ATTR_SET => self.convert_attr_set(node),
            SyntaxKind::NODE_LIST => self.convert_list(node),

            // Functions
            SyntaxKind::NODE_LAMBDA => self.convert_lambda(node),
            SyntaxKind::NODE_APPLY => self.convert_apply(node),

            // Control flow
            SyntaxKind::NODE_LET_IN => self.convert_let(node),
            SyntaxKind::NODE_IF_ELSE => self.convert_if(node),
            SyntaxKind::NODE_WITH => self.convert_with(node),
            SyntaxKind::NODE_ASSERT => self.convert_assert(node),

            // Operations
            SyntaxKind::NODE_BIN_OP => self.convert_binary_op(node),
            SyntaxKind::NODE_UNARY_OP => self.convert_unary_op(node),
            SyntaxKind::NODE_SELECT => self.convert_select(node),
            SyntaxKind::NODE_HAS_ATTR => self.convert_has_attr(node),

            // Root node - process its first child
            SyntaxKind::NODE_ROOT => {
                if let Some(child) = node.first_child() {
                    self.convert_node(&child)
                } else {
                    Err(NixDomainError::ParseError("Empty file".to_string()))
                }
            }

            _ => {
                // Check if this is a literal token
                if let Some(token) = node.first_token() {
                    let text = token.text();

                    // Try to parse as literal
                    if text == "true" {
                        return Ok(NixAst::Bool(true));
                    } else if text == "false" {
                        return Ok(NixAst::Bool(false));
                    } else if text == "null" {
                        return Ok(NixAst::Null);
                    } else if let Ok(int) = text.parse::<i64>() {
                        return Ok(NixAst::Integer(int));
                    } else if let Ok(float) = text.parse::<f64>() {
                        return Ok(NixAst::Float(float));
                    }
                }

                // Try to find a meaningful child node
                for child in node.children() {
                    if !matches!(
                        child.kind(),
                        SyntaxKind::TOKEN_WHITESPACE | SyntaxKind::TOKEN_COMMENT
                    ) {
                        return self.convert_node(&child);
                    }
                }

                // Check if this is an import (import is not a special node in newer rnix)
                if node.text().to_string().starts_with("import") {
                    return self.convert_import_expr(node);
                }

                // As a last resort, treat it as an identifier
                let text = node.text().to_string().trim().to_string();
                if !text.is_empty() {
                    return Ok(NixAst::Identifier(text));
                }

                Err(NixDomainError::ParseError(format!(
                    "Unsupported node kind: {:?}",
                    node.kind()
                )))
            }
        }
    }

    fn convert_literal(&self, node: &SyntaxNode) -> Result<NixAst> {
        let text = node.text().to_string();

        // Try to parse as different literal types
        if text == "true" {
            Ok(NixAst::Bool(true))
        } else if text == "false" {
            Ok(NixAst::Bool(false))
        } else if text == "null" {
            Ok(NixAst::Null)
        } else if let Ok(int) = text.parse::<i64>() {
            Ok(NixAst::Integer(int))
        } else if let Ok(float) = text.parse::<f64>() {
            Ok(NixAst::Float(float))
        } else {
            Err(NixDomainError::ParseError(format!(
                "Unknown literal: {text}"
            )))
        }
    }

    fn convert_string(&self, node: &SyntaxNode) -> Result<NixAst> {
        let text = node.text().to_string();

        // Handle different string types
        if text.starts_with("''") {
            // Indented string
            let content = text.trim_start_matches("''").trim_end_matches("''");
            Ok(NixAst::String(self.process_indented_string(content)))
        } else if text.starts_with('"') {
            // Regular string
            let content = text.trim_matches('"');
            Ok(NixAst::String(content.to_string()))
        } else if text.starts_with('/') || text.starts_with("./") || text.starts_with("~/") {
            // Path literal
            Ok(NixAst::Path(PathBuf::from(text)))
        } else {
            Ok(NixAst::String(text))
        }
    }

    fn convert_identifier(&self, node: &SyntaxNode) -> Result<NixAst> {
        let text = node.text().to_string();
        // Check for keywords
        match text.as_str() {
            "true" => Ok(NixAst::Bool(true)),
            "false" => Ok(NixAst::Bool(false)),
            "null" => Ok(NixAst::Null),
            _ => Ok(NixAst::Identifier(text)),
        }
    }

    fn convert_attr_set(&self, node: &SyntaxNode) -> Result<NixAst> {
        let mut bindings = Vec::new();
        let mut recursive = false;

        // Check if it's a recursive attribute set
        if let Some(first_token) = node.first_token() {
            if first_token.text() == "rec" {
                recursive = true;
            }
        }

        // Process bindings - in rnix 0.11, we need to look for different patterns
        for child in node.children() {
            // Check if this looks like a binding (has = in it)
            let child_text = child.text().to_string();
            if child_text.contains('=') && !child_text.starts_with("inherit") {
                if let Ok(binding) = self.convert_key_value(&child) {
                    bindings.push(binding);
                }
            } else if child_text.starts_with("inherit") {
                if let Ok(binding) = self.convert_inherit(&child) {
                    bindings.push(binding);
                }
            }
        }

        Ok(NixAst::AttrSet {
            recursive,
            bindings,
        })
    }

    fn convert_list(&self, node: &SyntaxNode) -> Result<NixAst> {
        let mut elements = Vec::new();

        for child in node.children() {
            if !matches!(
                child.kind(),
                SyntaxKind::TOKEN_WHITESPACE | SyntaxKind::TOKEN_COMMENT
            ) {
                elements.push(self.convert_node(&child)?);
            }
        }

        Ok(NixAst::List(elements))
    }

    fn convert_lambda(&self, node: &SyntaxNode) -> Result<NixAst> {
        let mut children = node.children();

        // First child is the parameter
        let param = if let Some(param_node) = children.next() {
            self.convert_function_param(&param_node)?
        } else {
            return Err(NixDomainError::ParseError(
                "Lambda missing parameter".to_string(),
            ));
        };

        // Skip the colon
        children.next();

        // Next is the body
        let body = if let Some(body_node) = children.next() {
            Box::new(self.convert_node(&body_node)?)
        } else {
            return Err(NixDomainError::ParseError(
                "Lambda missing body".to_string(),
            ));
        };

        Ok(NixAst::Function { param, body })
    }

    fn convert_apply(&self, node: &SyntaxNode) -> Result<NixAst> {
        let mut children = node.children();

        let function = if let Some(func_node) = children.next() {
            Box::new(self.convert_node(&func_node)?)
        } else {
            return Err(NixDomainError::ParseError(
                "Apply missing function".to_string(),
            ));
        };

        let argument = if let Some(arg_node) = children.next() {
            Box::new(self.convert_node(&arg_node)?)
        } else {
            return Err(NixDomainError::ParseError(
                "Apply missing argument".to_string(),
            ));
        };

        Ok(NixAst::Apply { function, argument })
    }

    fn convert_let(&self, node: &SyntaxNode) -> Result<NixAst> {
        let mut bindings = Vec::new();
        let mut body = None;
        let mut in_bindings = true;

        for child in node.children() {
            if child.kind() == SyntaxKind::TOKEN_IN {
                in_bindings = false;
                continue;
            }

            if in_bindings {
                // Similar to attr set, look for bindings
                let child_text = child.text().to_string();
                if child_text.contains('=') && !child_text.starts_with("inherit") {
                    if let Ok(binding) = self.convert_key_value(&child) {
                        bindings.push(binding);
                    }
                } else if child_text.starts_with("inherit") {
                    if let Ok(binding) = self.convert_inherit(&child) {
                        bindings.push(binding);
                    }
                }
            } else if body.is_none() {
                body = Some(Box::new(self.convert_node(&child)?));
            }
        }

        let body =
            body.ok_or_else(|| NixDomainError::ParseError("Let missing body".to_string()))?;

        Ok(NixAst::Let { bindings, body })
    }

    fn convert_if(&self, node: &SyntaxNode) -> Result<NixAst> {
        let mut children = node.children();

        // Skip 'if' token
        children.next();

        let condition = if let Some(cond_node) = children.next() {
            Box::new(self.convert_node(&cond_node)?)
        } else {
            return Err(NixDomainError::ParseError(
                "If missing condition".to_string(),
            ));
        };

        // Skip 'then' token
        children.next();

        let then_branch = if let Some(then_node) = children.next() {
            Box::new(self.convert_node(&then_node)?)
        } else {
            return Err(NixDomainError::ParseError(
                "If missing then branch".to_string(),
            ));
        };

        // Skip 'else' token
        children.next();

        let else_branch = if let Some(else_node) = children.next() {
            Box::new(self.convert_node(&else_node)?)
        } else {
            return Err(NixDomainError::ParseError(
                "If missing else branch".to_string(),
            ));
        };

        Ok(NixAst::If {
            condition,
            then_branch,
            else_branch,
        })
    }

    fn convert_with(&self, node: &SyntaxNode) -> Result<NixAst> {
        let mut children = node.children();

        // Skip 'with' token
        children.next();

        let namespace = if let Some(ns_node) = children.next() {
            Box::new(self.convert_node(&ns_node)?)
        } else {
            return Err(NixDomainError::ParseError(
                "With missing namespace".to_string(),
            ));
        };

        // Skip semicolon
        children.next();

        let body = if let Some(body_node) = children.next() {
            Box::new(self.convert_node(&body_node)?)
        } else {
            return Err(NixDomainError::ParseError("With missing body".to_string()));
        };

        Ok(NixAst::With { namespace, body })
    }

    fn convert_assert(&self, node: &SyntaxNode) -> Result<NixAst> {
        let mut children = node.children();

        // Skip 'assert' token
        children.next();

        let condition = if let Some(cond_node) = children.next() {
            Box::new(self.convert_node(&cond_node)?)
        } else {
            return Err(NixDomainError::ParseError(
                "Assert missing condition".to_string(),
            ));
        };

        // Skip semicolon
        children.next();

        let body = if let Some(body_node) = children.next() {
            Box::new(self.convert_node(&body_node)?)
        } else {
            return Err(NixDomainError::ParseError(
                "Assert missing body".to_string(),
            ));
        };

        Ok(NixAst::Assert { condition, body })
    }

    fn convert_binary_op(&self, node: &SyntaxNode) -> Result<NixAst> {
        let mut children = node.children();

        let left = if let Some(left_node) = children.next() {
            Box::new(self.convert_node(&left_node)?)
        } else {
            return Err(NixDomainError::ParseError(
                "Binary op missing left operand".to_string(),
            ));
        };

        let op = if let Some(op_token) = children.next() {
            self.parse_binary_operator(&op_token.text().to_string())?
        } else {
            return Err(NixDomainError::ParseError(
                "Binary op missing operator".to_string(),
            ));
        };

        let right = if let Some(right_node) = children.next() {
            Box::new(self.convert_node(&right_node)?)
        } else {
            return Err(NixDomainError::ParseError(
                "Binary op missing right operand".to_string(),
            ));
        };

        Ok(NixAst::BinaryOp { op, left, right })
    }

    fn convert_unary_op(&self, node: &SyntaxNode) -> Result<NixAst> {
        let mut children = node.children();

        let op = if let Some(op_token) = children.next() {
            self.parse_unary_operator(&op_token.text().to_string())?
        } else {
            return Err(NixDomainError::ParseError(
                "Unary op missing operator".to_string(),
            ));
        };

        let operand = if let Some(operand_node) = children.next() {
            Box::new(self.convert_node(&operand_node)?)
        } else {
            return Err(NixDomainError::ParseError(
                "Unary op missing operand".to_string(),
            ));
        };

        Ok(NixAst::UnaryOp { op, operand })
    }

    fn convert_select(&self, node: &SyntaxNode) -> Result<NixAst> {
        let mut children = node.children();

        let expr = if let Some(expr_node) = children.next() {
            Box::new(self.convert_node(&expr_node)?)
        } else {
            return Err(NixDomainError::ParseError(
                "Select missing expression".to_string(),
            ));
        };

        // Skip dot
        children.next();

        let attr_path = if let Some(path_node) = children.next() {
            self.parse_attr_path(&path_node)?
        } else {
            return Err(NixDomainError::ParseError(
                "Select missing attribute path".to_string(),
            ));
        };

        // Check for default value (or syntax)
        let default = if children.any(|c| c.text() == "or") {
            children
                .next()
                .map(|n| Box::new(self.convert_node(&n).unwrap()))
        } else {
            None
        };

        Ok(NixAst::Select {
            expr,
            attr_path,
            default,
        })
    }

    fn convert_has_attr(&self, node: &SyntaxNode) -> Result<NixAst> {
        let mut children = node.children();

        let expr = if let Some(expr_node) = children.next() {
            Box::new(self.convert_node(&expr_node)?)
        } else {
            return Err(NixDomainError::ParseError(
                "HasAttr missing expression".to_string(),
            ));
        };

        // Skip ?
        children.next();

        let attr_path = if let Some(path_node) = children.next() {
            self.parse_attr_path(&path_node)?
        } else {
            return Err(NixDomainError::ParseError(
                "HasAttr missing attribute path".to_string(),
            ));
        };

        Ok(NixAst::HasAttr { expr, attr_path })
    }

    fn convert_import_expr(&self, node: &SyntaxNode) -> Result<NixAst> {
        let mut children = node.children();

        // Skip 'import' token
        children.next();

        let path = if let Some(path_node) = children.next() {
            Box::new(self.convert_node(&path_node)?)
        } else {
            return Err(NixDomainError::ParseError(
                "Import missing path".to_string(),
            ));
        };

        Ok(NixAst::Import(path))
    }

    // Helper methods

    fn convert_key_value(&self, node: &SyntaxNode) -> Result<Binding> {
        // Parse a key = value binding
        let text = node.text().to_string();
        let parts: Vec<&str> = text.split('=').collect();

        if parts.len() != 2 {
            return Err(NixDomainError::ParseError(
                "Invalid key-value binding".to_string(),
            ));
        }

        let key = parts[0].trim();
        let value_str = parts[1].trim().trim_end_matches(';').trim();

        // Simple parsing for common cases
        let value = if value_str.starts_with('"') && value_str.ends_with('"') {
            NixAst::String(value_str.trim_matches('"').to_string())
        } else if let Ok(num) = value_str.parse::<i64>() {
            NixAst::Integer(num)
        } else if value_str == "true" {
            NixAst::Bool(true)
        } else if value_str == "false" {
            NixAst::Bool(false)
        } else {
            NixAst::Identifier(value_str.to_string())
        };

        Ok(Binding {
            attr_path: AttrPath {
                segments: vec![AttrPathSegment::Identifier(key.to_string())],
            },
            value: BindingValue::Value(value),
        })
    }

    fn convert_inherit(&self, _node: &SyntaxNode) -> Result<Binding> {
        // Handle inherit binding
        let (from, attrs) = self.parse_inherit(_node)?;
        Ok(Binding {
            attr_path: AttrPath { segments: vec![] }, // Inherit doesn't have a path
            value: BindingValue::Inherit { from, attrs },
        })
    }

    fn convert_function_param(&self, node: &SyntaxNode) -> Result<FunctionParam> {
        match node.kind() {
            SyntaxKind::NODE_IDENT => Ok(FunctionParam::Identifier(node.text().to_string())),
            SyntaxKind::NODE_PATTERN => {
                // Parse pattern { a, b ? default, ... }
                let (fields, bind, ellipsis) = self.parse_pattern(node)?;
                Ok(FunctionParam::Pattern {
                    fields,
                    bind,
                    ellipsis,
                })
            }
            _ => Err(NixDomainError::ParseError(
                "Invalid function parameter".to_string(),
            )),
        }
    }

    fn parse_pattern(
        &self,
        _node: &SyntaxNode,
    ) -> Result<(Vec<PatternField>, Option<String>, bool)> {
        let fields = Vec::new();
        let bind = None;
        let ellipsis = false;

        // TODO: Implement pattern parsing
        // This is complex and requires careful handling of the pattern syntax

        Ok((fields, bind, ellipsis))
    }

    fn parse_inherit(&self, _node: &SyntaxNode) -> Result<(Option<NixAst>, Vec<String>)> {
        let from = None;
        let attrs = Vec::new();

        // TODO: Implement inherit parsing

        Ok((from, attrs))
    }

    fn parse_attr_path(&self, node: &SyntaxNode) -> Result<AttrPath> {
        let segments = vec![AttrPathSegment::Identifier(node.text().to_string())];
        Ok(AttrPath { segments })
    }

    fn parse_binary_operator(&self, text: &str) -> Result<BinaryOperator> {
        match text {
            "+" => Ok(BinaryOperator::Add),
            "-" => Ok(BinaryOperator::Subtract),
            "*" => Ok(BinaryOperator::Multiply),
            "/" => Ok(BinaryOperator::Divide),
            "==" => Ok(BinaryOperator::Equal),
            "!=" => Ok(BinaryOperator::NotEqual),
            "<" => Ok(BinaryOperator::Less),
            "<=" => Ok(BinaryOperator::LessEqual),
            ">" => Ok(BinaryOperator::Greater),
            ">=" => Ok(BinaryOperator::GreaterEqual),
            "&&" => Ok(BinaryOperator::And),
            "||" => Ok(BinaryOperator::Or),
            "->" => Ok(BinaryOperator::Implies),
            "++" => Ok(BinaryOperator::Concat),
            "//" => Ok(BinaryOperator::Update),
            _ => Err(NixDomainError::ParseError(format!(
                "Unknown binary operator: {text}"
            ))),
        }
    }

    fn parse_unary_operator(&self, text: &str) -> Result<UnaryOperator> {
        match text {
            "!" => Ok(UnaryOperator::Not),
            "-" => Ok(UnaryOperator::Negate),
            _ => Err(NixDomainError::ParseError(format!(
                "Unknown unary operator: {text}"
            ))),
        }
    }

    fn process_indented_string(&self, content: &str) -> String {
        // TODO: Properly process indented strings
        content.to_string()
    }

    fn node_location(&self, node: &SyntaxNode) -> Location {
        let range = node.text_range();
        Location {
            file: self.current_file.clone(),
            line: 0,   // TODO: Calculate line number
            column: 0, // TODO: Calculate column
            offset: range.start().into(),
            length: range.len().into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_expressions() {
        let parser = AdvancedParser::new();

        // Test integer
        let ast = parser.parse_string("42").unwrap();
        match ast.ast {
            NixAst::Integer(n) => assert_eq!(n, 42),
            _ => panic!("Expected integer"),
        }

        // Test string
        let ast = parser.parse_string(r#""hello world""#).unwrap();
        match ast.ast {
            NixAst::String(s) => assert_eq!(s, "hello world"),
            _ => panic!("Expected string"),
        }

        // Test boolean
        let ast = parser.parse_string("true").unwrap();
        match ast.ast {
            NixAst::Bool(b) => assert!(b),
            _ => panic!("Expected boolean"),
        }
    }

    #[test]
    fn test_parse_attribute_set() {
        let parser = AdvancedParser::new();
        let ast = parser
            .parse_string(r#"{ foo = 42; bar = "hello"; }"#)
            .unwrap();

        match ast.ast {
            NixAst::AttrSet {
                recursive,
                bindings,
            } => {
                assert!(!recursive);
                assert_eq!(bindings.len(), 2);
            }
            _ => panic!("Expected attribute set"),
        }
    }

    #[test]
    fn test_parse_list() {
        let parser = AdvancedParser::new();
        let ast = parser.parse_string(r#"[ 1 2 3 "foo" ]"#).unwrap();

        match ast.ast {
            NixAst::List(elements) => {
                assert_eq!(elements.len(), 4);
            }
            _ => panic!("Expected list"),
        }
    }
}

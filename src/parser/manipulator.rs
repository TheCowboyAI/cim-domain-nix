//! AST manipulation utilities for modifying Nix expressions

use super::ast::{NixAst, AttrPath, AttrPathSegment, Binding, BindingValue, FunctionParam, BinaryOperator};
use crate::Result;

/// AST manipulator for transforming Nix expressions
pub struct AstManipulator;

impl AstManipulator {
    /// Add a new attribute to an attribute set
    pub fn add_attribute(
        ast: &mut NixAst,
        path: Vec<&str>,
        value: NixAst,
    ) -> Result<()> {
        match ast {
            NixAst::AttrSet { bindings, .. } => {
                let attr_path = AttrPath {
                    segments: path.into_iter()
                        .map(|s| AttrPathSegment::Identifier(s.to_string()))
                        .collect(),
                };
                
                bindings.push(Binding {
                    attr_path,
                    value: BindingValue::Value(value),
                });
                
                Ok(())
            }
            _ => Err(crate::NixDomainError::ParseError(
                "Can only add attributes to attribute sets".to_string()
            ))
        }
    }

    /// Remove an attribute from an attribute set
    pub fn remove_attribute(
        ast: &mut NixAst,
        path: Vec<&str>,
    ) -> Result<Option<NixAst>> {
        match ast {
            NixAst::AttrSet { bindings, .. } => {
                let pos = bindings.iter().position(|b| {
                    Self::attr_path_matches(&b.attr_path, &path)
                });
                
                if let Some(pos) = pos {
                    let binding = bindings.remove(pos);
                    match binding.value {
                        BindingValue::Value(v) => Ok(Some(v)),
                        _ => Ok(None),
                    }
                } else {
                    Ok(None)
                }
            }
            _ => Err(crate::NixDomainError::ParseError(
                "Can only remove attributes from attribute sets".to_string()
            ))
        }
    }

    /// Update an existing attribute value
    pub fn update_attribute(
        ast: &mut NixAst,
        path: Vec<&str>,
        new_value: NixAst,
    ) -> Result<Option<NixAst>> {
        match ast {
            NixAst::AttrSet { bindings, .. } => {
                for binding in bindings.iter_mut() {
                    if Self::attr_path_matches(&binding.attr_path, &path) {
                        match &mut binding.value {
                            BindingValue::Value(v) => {
                                let old_value = std::mem::replace(v, new_value);
                                return Ok(Some(old_value));
                            }
                            _ => return Ok(None),
                        }
                    }
                }
                Ok(None)
            }
            _ => Err(crate::NixDomainError::ParseError(
                "Can only update attributes in attribute sets".to_string()
            ))
        }
    }

    /// Get an attribute value by path
    pub fn get_attribute<'a>(
        ast: &'a NixAst,
        path: Vec<&str>,
    ) -> Option<&'a NixAst> {
        match ast {
            NixAst::AttrSet { bindings, .. } => {
                for binding in bindings {
                    if Self::attr_path_matches(&binding.attr_path, &path) {
                        match &binding.value {
                            BindingValue::Value(v) => return Some(v),
                            _ => return None,
                        }
                    }
                }
                None
            }
            _ => None
        }
    }

    /// Add an element to a list
    pub fn add_list_element(
        ast: &mut NixAst,
        element: NixAst,
    ) -> Result<()> {
        match ast {
            NixAst::List(elements) => {
                elements.push(element);
                Ok(())
            }
            _ => Err(crate::NixDomainError::ParseError(
                "Can only add elements to lists".to_string()
            ))
        }
    }

    /// Remove an element from a list by index
    pub fn remove_list_element(
        ast: &mut NixAst,
        index: usize,
    ) -> Result<Option<NixAst>> {
        match ast {
            NixAst::List(elements) => {
                if index < elements.len() {
                    Ok(Some(elements.remove(index)))
                } else {
                    Ok(None)
                }
            }
            _ => Err(crate::NixDomainError::ParseError(
                "Can only remove elements from lists".to_string()
            ))
        }
    }

    /// Transform nodes matching a predicate (renamed from transform)
    pub fn transform_nodes<F>(ast: &mut NixAst, transformer: &F)
    where
        F: Fn(&mut NixAst) -> Option<NixAst>,
    {
        // First try to transform the current node
        if let Some(new_ast) = transformer(ast) {
            *ast = new_ast;
            return;
        }

        // Then recurse into children
        match ast {
            NixAst::AttrSet { bindings, .. } => {
                for binding in bindings {
                    if let BindingValue::Value(ref mut v) = binding.value {
                        Self::transform_nodes(v, transformer);
                    }
                }
            }
            NixAst::List(elements) => {
                for elem in elements {
                    Self::transform_nodes(elem, transformer);
                }
            }
            NixAst::Function { body, .. } => {
                Self::transform_nodes(body, transformer);
            }
            NixAst::Apply { function, argument } => {
                Self::transform_nodes(function, transformer);
                Self::transform_nodes(argument, transformer);
            }
            NixAst::Let { bindings, body } => {
                for binding in bindings {
                    if let BindingValue::Value(ref mut v) = binding.value {
                        Self::transform_nodes(v, transformer);
                    }
                }
                Self::transform_nodes(body, transformer);
            }
            NixAst::If { condition, then_branch, else_branch } => {
                Self::transform_nodes(condition, transformer);
                Self::transform_nodes(then_branch, transformer);
                Self::transform_nodes(else_branch, transformer);
            }
            NixAst::With { namespace, body } => {
                Self::transform_nodes(namespace, transformer);
                Self::transform_nodes(body, transformer);
            }
            NixAst::Assert { condition, body } => {
                Self::transform_nodes(condition, transformer);
                Self::transform_nodes(body, transformer);
            }
            NixAst::BinaryOp { left, right, .. } => {
                Self::transform_nodes(left, transformer);
                Self::transform_nodes(right, transformer);
            }
            NixAst::UnaryOp { operand, .. } => {
                Self::transform_nodes(operand, transformer);
            }
            NixAst::Select { expr, default, .. } => {
                Self::transform_nodes(expr, transformer);
                if let Some(d) = default {
                    Self::transform_nodes(d, transformer);
                }
            }
            NixAst::HasAttr { expr, .. } => {
                Self::transform_nodes(expr, transformer);
            }
            NixAst::Import(path) => {
                Self::transform_nodes(path, transformer);
            }
            _ => {}
        }
    }

    /// Find all nodes matching a predicate
    pub fn find_nodes<'a, F>(ast: &'a NixAst, predicate: &F) -> Vec<&'a NixAst>
    where
        F: Fn(&NixAst) -> bool,
    {
        let mut results = Vec::new();
        Self::find_nodes_recursive(ast, predicate, &mut results);
        results
    }

    fn find_nodes_recursive<'a, F>(
        ast: &'a NixAst,
        predicate: &F,
        results: &mut Vec<&'a NixAst>,
    ) where
        F: Fn(&NixAst) -> bool,
    {
        if predicate(ast) {
            results.push(ast);
        }

        match ast {
            NixAst::AttrSet { bindings, .. } => {
                for binding in bindings {
                    if let BindingValue::Value(value) = &binding.value {
                        Self::find_nodes_recursive(value, predicate, results);
                    }
                }
            }
            NixAst::List(items) => {
                for item in items {
                    Self::find_nodes_recursive(item, predicate, results);
                }
            }
            NixAst::Function { body, .. } => {
                Self::find_nodes_recursive(body, predicate, results);
            }
            NixAst::Apply { function, argument } => {
                Self::find_nodes_recursive(function, predicate, results);
                Self::find_nodes_recursive(argument, predicate, results);
            }
            NixAst::Let { bindings, body } => {
                for binding in bindings {
                    if let BindingValue::Value(value) = &binding.value {
                        Self::find_nodes_recursive(value, predicate, results);
                    }
                }
                Self::find_nodes_recursive(body, predicate, results);
            }
            NixAst::If { condition, then_branch, else_branch } => {
                Self::find_nodes_recursive(condition, predicate, results);
                Self::find_nodes_recursive(then_branch, predicate, results);
                Self::find_nodes_recursive(else_branch, predicate, results);
            }
            NixAst::With { namespace, body } => {
                Self::find_nodes_recursive(namespace, predicate, results);
                Self::find_nodes_recursive(body, predicate, results);
            }
            NixAst::Assert { condition, body } => {
                Self::find_nodes_recursive(condition, predicate, results);
                Self::find_nodes_recursive(body, predicate, results);
            }
            NixAst::BinaryOp { left, right, .. } => {
                Self::find_nodes_recursive(left, predicate, results);
                Self::find_nodes_recursive(right, predicate, results);
            }
            NixAst::UnaryOp { operand, .. } => {
                Self::find_nodes_recursive(operand, predicate, results);
            }
            NixAst::Select { expr, default, .. } => {
                Self::find_nodes_recursive(expr, predicate, results);
                if let Some(default) = default {
                    Self::find_nodes_recursive(default, predicate, results);
                }
            }
            NixAst::HasAttr { expr, .. } => {
                Self::find_nodes_recursive(expr, predicate, results);
            }
            _ => {} // Leaf nodes
        }
    }

    /// Replace all nodes matching a predicate
    pub fn replace_nodes<F, G>(ast: &mut NixAst, predicate: F, replacer: G) -> Result<usize>
    where
        F: Fn(&NixAst) -> bool,
        G: Fn(&NixAst) -> Option<NixAst>,
    {
        let mut count = 0;
        Self::replace_nodes_recursive(ast, &predicate, &replacer, &mut count)?;
        Ok(count)
    }

    fn replace_nodes_recursive<F, G>(
        ast: &mut NixAst,
        predicate: &F,
        replacer: &G,
        count: &mut usize,
    ) -> Result<()>
    where
        F: Fn(&NixAst) -> bool,
        G: Fn(&NixAst) -> Option<NixAst>,
    {
        // Check if current node should be replaced
        if predicate(ast) {
            if let Some(replacement) = replacer(ast) {
                *ast = replacement;
                *count += 1;
                return Ok(());
            }
        }

        // Recursively process children
        match ast {
            NixAst::AttrSet { bindings, .. } => {
                for binding in bindings {
                    match &mut binding.value {
                        BindingValue::Value(v) => {
                            Self::replace_nodes_recursive(v, predicate, replacer, count)?;
                        }
                        BindingValue::Inherit { from: Some(f), .. } => {
                            Self::replace_nodes_recursive(f, predicate, replacer, count)?;
                        }
                        _ => {}
                    }
                }
            }
            NixAst::List(elements) => {
                for element in elements {
                    Self::replace_nodes_recursive(element, predicate, replacer, count)?;
                }
            }
            NixAst::Function { body, .. } => {
                Self::replace_nodes_recursive(body, predicate, replacer, count)?;
            }
            NixAst::Apply { function, argument } => {
                Self::replace_nodes_recursive(function, predicate, replacer, count)?;
                Self::replace_nodes_recursive(argument, predicate, replacer, count)?;
            }
            NixAst::Let { bindings, body } => {
                for binding in bindings {
                    if let BindingValue::Value(v) = &mut binding.value {
                        Self::replace_nodes_recursive(v, predicate, replacer, count)?;
                    }
                }
                Self::replace_nodes_recursive(body, predicate, replacer, count)?;
            }
            NixAst::If { condition, then_branch, else_branch } => {
                Self::replace_nodes_recursive(condition, predicate, replacer, count)?;
                Self::replace_nodes_recursive(then_branch, predicate, replacer, count)?;
                Self::replace_nodes_recursive(else_branch, predicate, replacer, count)?;
            }
            NixAst::With { namespace, body } => {
                Self::replace_nodes_recursive(namespace, predicate, replacer, count)?;
                Self::replace_nodes_recursive(body, predicate, replacer, count)?;
            }
            NixAst::Assert { condition, body } => {
                Self::replace_nodes_recursive(condition, predicate, replacer, count)?;
                Self::replace_nodes_recursive(body, predicate, replacer, count)?;
            }
            NixAst::BinaryOp { left, right, .. } => {
                Self::replace_nodes_recursive(left, predicate, replacer, count)?;
                Self::replace_nodes_recursive(right, predicate, replacer, count)?;
            }
            NixAst::UnaryOp { operand, .. } => {
                Self::replace_nodes_recursive(operand, predicate, replacer, count)?;
            }
            NixAst::Select { expr, default, .. } => {
                Self::replace_nodes_recursive(expr, predicate, replacer, count)?;
                if let Some(d) = default {
                    Self::replace_nodes_recursive(d, predicate, replacer, count)?;
                }
            }
            NixAst::HasAttr { expr, .. } => {
                Self::replace_nodes_recursive(expr, predicate, replacer, count)?;
            }
            NixAst::Import(path) => {
                Self::replace_nodes_recursive(path, predicate, replacer, count)?;
            }
            _ => {}
        }

        Ok(())
    }

    // Helper function to check if an attribute path matches
    fn attr_path_matches(path: &AttrPath, target: &[&str]) -> bool {
        if path.segments.len() != target.len() {
            return false;
        }

        path.segments.iter().zip(target.iter()).all(|(segment, target_part)| {
            match segment {
                AttrPathSegment::Identifier(name) => name == target_part,
                _ => false,
            }
        })
    }

    /// Helper method for building simple functions
    pub fn simple_function(param: &str, body: NixAst) -> NixAst {
        NixAst::Function {
            param: FunctionParam::Identifier(param.to_string()),
            body: Box::new(body),
        }
    }

    /// Helper method for building if expressions
    pub fn if_expr(condition: NixAst, then_branch: NixAst, else_branch: NixAst) -> NixAst {
        NixAst::If {
            condition: Box::new(condition),
            then_branch: Box::new(then_branch),
            else_branch: Box::new(else_branch),
        }
    }

    /// Helper method for building binary operations
    pub fn binary_op(op: BinaryOperator, left: NixAst, right: NixAst) -> NixAst {
        NixAst::BinaryOp {
            op,
            left: Box::new(left),
            right: Box::new(right),
        }
    }
}

/// Builder for constructing Nix AST nodes
pub struct AstBuilder {
    current: Option<NixAst>,
}

impl Default for AstBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl AstBuilder {
    /// Create a new builder
    pub fn new() -> Self {
        Self { current: None }
    }

    /// Start building an attribute set
    pub fn attr_set(mut self) -> Self {
        self.current = Some(NixAst::AttrSet {
            recursive: false,
            bindings: Vec::new(),
        });
        self
    }

    /// Add an attribute to the current attribute set
    pub fn add_attr(mut self, name: &str, value: NixAst) -> Self {
        if let Some(NixAst::AttrSet { bindings, .. }) = &mut self.current {
            bindings.push(Binding {
                attr_path: AttrPath {
                    segments: vec![AttrPathSegment::Identifier(name.to_string())],
                },
                value: BindingValue::Value(value),
            });
        }
        self
    }

    /// Build the final AST
    pub fn build(self) -> NixAst {
        self.current.unwrap_or(NixAst::Null)
    }

    /// Helper method for building simple functions
    pub fn simple_function(param: &str, body: NixAst) -> NixAst {
        NixAst::Function {
            param: FunctionParam::Identifier(param.to_string()),
            body: Box::new(body),
        }
    }

    /// Helper method for building if expressions
    pub fn if_expr(condition: NixAst, then_branch: NixAst, else_branch: NixAst) -> NixAst {
        NixAst::If {
            condition: Box::new(condition),
            then_branch: Box::new(then_branch),
            else_branch: Box::new(else_branch),
        }
    }

    /// Helper method for building binary operations
    pub fn binary_op(op: BinaryOperator, left: NixAst, right: NixAst) -> NixAst {
        NixAst::BinaryOp {
            op,
            left: Box::new(left),
            right: Box::new(right),
        }
    }

    /// Create a string literal
    pub fn string(s: &str) -> NixAst {
        NixAst::String(s.to_string())
    }

    /// Create an integer literal
    pub fn integer(i: i64) -> NixAst {
        NixAst::Integer(i)
    }

    /// Create a boolean literal
    pub fn bool(b: bool) -> NixAst {
        NixAst::Bool(b)
    }

    /// Create an attribute set (static method)
    pub fn attr_set_with(bindings: Vec<Binding>) -> NixAst {
        NixAst::AttrSet {
            recursive: false,
            bindings,
        }
    }

    /// Create a list (static method)
    pub fn list(elements: Vec<NixAst>) -> NixAst {
        NixAst::List(elements)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_attribute() {
        let mut ast = NixAst::AttrSet {
            recursive: false,
            bindings: Vec::new(),
        };

        AstManipulator::add_attribute(
            &mut ast,
            vec!["foo"],
            NixAst::String("bar".to_string()),
        ).unwrap();

        let value = AstManipulator::get_attribute(&ast, vec!["foo"]);
        assert!(value.is_some());
    }

    #[test]
    fn test_update_attribute() {
        let mut ast = NixAst::AttrSet {
            recursive: false,
            bindings: Vec::new(),
        };

        AstManipulator::add_attribute(
            &mut ast,
            vec!["foo"],
            NixAst::Integer(42),
        ).unwrap();

        let old = AstManipulator::update_attribute(
            &mut ast,
            vec!["foo"],
            NixAst::Integer(100),
        ).unwrap();

        assert!(matches!(old, Some(NixAst::Integer(42))));
        let value = AstManipulator::get_attribute(&ast, vec!["foo"]);
        assert!(matches!(value, Some(NixAst::Integer(100))));
    }

    #[test]
    fn test_find_nodes() {
        let ast = NixAst::AttrSet {
            recursive: false,
            bindings: vec![
                Binding {
                    attr_path: AttrPath {
                        segments: vec![AttrPathSegment::Identifier("x".to_string())],
                    },
                    value: BindingValue::Value(NixAst::Integer(42)),
                },
                Binding {
                    attr_path: AttrPath {
                        segments: vec![AttrPathSegment::Identifier("y".to_string())],
                    },
                    value: BindingValue::Value(NixAst::Integer(100)),
                },
                Binding {
                    attr_path: AttrPath {
                        segments: vec![AttrPathSegment::Identifier("z".to_string())],
                    },
                    value: BindingValue::Value(NixAst::String("test".to_string())),
                },
            ],
        };

        let integers = AstManipulator::find_nodes(&ast, &|node| {
            matches!(node, NixAst::Integer(_))
        });

        assert_eq!(integers.len(), 2);
    }

    #[test]
    fn test_replace_nodes() {
        let mut ast = AstBuilder::list(vec![
            NixAst::Integer(1),
            NixAst::Integer(2),
            NixAst::Integer(3),
        ]);

        let count = AstManipulator::replace_nodes(
            &mut ast,
            |node| matches!(node, NixAst::Integer(2)),
            |_| Some(NixAst::Integer(42)),
        ).unwrap();

        assert_eq!(count, 1);
        
        match &ast {
            NixAst::List(elements) => {
                assert!(matches!(elements[1], NixAst::Integer(42)));
            }
            _ => panic!("Expected list"),
        }
    }

    #[test]
    fn test_transform_nodes() {
        let mut ast = AstBuilder::list(vec![
            NixAst::Integer(1),
            NixAst::Integer(2),
            NixAst::Integer(3),
        ]);

        AstManipulator::transform_nodes(&mut ast, &|node| {
            match node {
                NixAst::Integer(n) => Some(NixAst::Integer(*n * 2)),
                _ => None,
            }
        });

        match ast {
            NixAst::List(elements) => {
                assert_eq!(elements.len(), 3);
                assert!(matches!(elements[0], NixAst::Integer(2)));
                assert!(matches!(elements[1], NixAst::Integer(4)));
                assert!(matches!(elements[2], NixAst::Integer(6)));
            }
            _ => panic!("Expected list"),
        }
    }
} 
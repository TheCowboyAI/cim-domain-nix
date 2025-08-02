//! Tests for the Nix AST parser and manipulation utilities
//!
//! ```mermaid
//! graph TD
//!     A[Parse Nix String] --> B{Parse Success?}
//!     B -->|Yes| C[Convert to AST]
//!     B -->|No| D[Return Error]
//!     C --> E[Manipulate AST]
//!     E --> F[Query AST]
//!     E --> G[Transform AST]
//!     E --> H[Build New AST]
//! ```

use super::*;
use crate::parser::advanced::AdvancedParser;
use crate::parser::ast::*;
use crate::parser::manipulator::{AstBuilder, AstManipulator};

#[cfg(test)]
mod ast_parser_tests {
    use super::*;
    use std::path::PathBuf;

    /// Test parsing simple literals
    ///
    /// ```mermaid
    /// graph LR
    ///     A[String Input] --> B[Parse]
    ///     B --> C[AST Node]
    ///     C --> D{Validate Type}
    ///     D -->|Integer| E[Check Value]
    ///     D -->|String| F[Check Content]
    ///     D -->|Bool| G[Check Boolean]
    /// ```
    #[test]
    fn test_parse_literals() {
        let parser = AdvancedParser::new();

        // Test integer
        let int_ast = parser.parse_string("42").unwrap();
        match &int_ast.ast {
            NixAst::Integer(42) => {} // AdvancedParser correctly returns Integer
            _ => panic!("Expected Integer(42), got {:?}", int_ast.ast),
        }

        // Test string
        let str_ast = parser.parse_string(r#""hello world""#).unwrap();
        match &str_ast.ast {
            NixAst::String(s) => assert_eq!(s, "hello world"),
            _ => panic!("Expected String, got {:?}", str_ast.ast),
        }

        // Test boolean
        let bool_ast = parser.parse_string("true").unwrap();
        match &bool_ast.ast {
            NixAst::Bool(true) => {}
            _ => panic!("Expected Bool(true), got {:?}", bool_ast.ast),
        }
    }

    /// Test parsing attribute sets
    ///
    /// ```mermaid
    /// graph TD
    ///     A[AttrSet String] --> B[Parse]
    ///     B --> C[AST AttrSet]
    ///     C --> D[Check Bindings]
    ///     D --> E[Validate Keys]
    ///     D --> F[Validate Values]
    /// ```
    #[test]
    fn test_parse_attribute_sets() {
        let parser = AdvancedParser::new();

        let attrset_content = r#"{ 
            name = "test"; 
            version = "1.0.0";
            enabled = true;
        }"#;

        let parsed = parser.parse_string(attrset_content).unwrap();

        // AdvancedParser should return a proper AttrSet
        match &parsed.ast {
            NixAst::AttrSet {
                bindings,
                recursive,
            } => {
                assert!(!recursive);
                assert!(bindings.len() >= 3); // At least 3 bindings

                // Check that we have the expected attributes
                let has_name = bindings.iter().any(|b| {
                    matches!(
                        b.attr_path.segments.first(),
                        Some(AttrPathSegment::Identifier(s)) if s == "name"
                    )
                });
                let has_version = bindings.iter().any(|b| {
                    matches!(
                        b.attr_path.segments.first(),
                        Some(AttrPathSegment::Identifier(s)) if s == "version"
                    )
                });
                let has_enabled = bindings.iter().any(|b| {
                    matches!(
                        b.attr_path.segments.first(),
                        Some(AttrPathSegment::Identifier(s)) if s == "enabled"
                    )
                });

                assert!(has_name, "Missing 'name' attribute");
                assert!(has_version, "Missing 'version' attribute");
                assert!(has_enabled, "Missing 'enabled' attribute");
            }
            _ => panic!("Expected AttrSet, got {:?}", parsed.ast),
        }
    }

    /// Test AST builder functionality
    ///
    /// ```mermaid
    /// graph LR
    ///     A[Builder] --> B[Add Attrs]
    ///     B --> C[Add Values]
    ///     C --> D[Build AST]
    ///     D --> E[Validate Structure]
    /// ```
    #[test]
    fn test_ast_builder() {
        let ast = AstBuilder::new()
            .attr_set()
            .add_attr("name", NixAst::String("my-package".to_string()))
            .add_attr("version", NixAst::String("1.0.0".to_string()))
            .add_attr(
                "meta",
                AstBuilder::new()
                    .attr_set()
                    .add_attr("description", NixAst::String("A test package".to_string()))
                    .add_attr("license", NixAst::String("MIT".to_string()))
                    .build(),
            )
            .build();

        match &ast {
            NixAst::AttrSet {
                bindings,
                recursive,
            } => {
                assert!(!recursive);
                assert_eq!(bindings.len(), 3);

                // Check name attribute
                let name_binding = bindings
                    .iter()
                    .find(|b| {
                        matches!(
                            b.attr_path.segments.first(),
                            Some(AttrPathSegment::Identifier(s)) if s == "name"
                        )
                    })
                    .expect("name attribute not found");

                match &name_binding.value {
                    BindingValue::Value(NixAst::String(s)) => {
                        assert_eq!(s, "my-package");
                    }
                    _ => panic!("Expected string value for name"),
                }

                // Check nested meta attribute
                let meta_binding = bindings
                    .iter()
                    .find(|b| {
                        matches!(
                            b.attr_path.segments.first(),
                            Some(AttrPathSegment::Identifier(s)) if s == "meta"
                        )
                    })
                    .expect("meta attribute not found");

                match &meta_binding.value {
                    BindingValue::Value(NixAst::AttrSet {
                        bindings: meta_bindings,
                        ..
                    }) => {
                        assert_eq!(meta_bindings.len(), 2);
                    }
                    _ => panic!("Expected attribute set for meta"),
                }
            }
            _ => panic!("Expected attribute set"),
        }
    }

    /// Test AST manipulation - finding nodes
    ///
    /// ```mermaid
    /// graph TD
    ///     A[AST Tree] --> B[Find Nodes]
    ///     B --> C{Match Predicate?}
    ///     C -->|Yes| D[Add to Results]
    ///     C -->|No| E[Skip]
    ///     D --> F[Return Results]
    /// ```
    #[test]
    fn test_find_nodes() {
        let ast = AstBuilder::new()
            .attr_set()
            .add_attr("name", NixAst::String("test".to_string()))
            .add_attr(
                "deps",
                NixAst::List(vec![
                    NixAst::String("dep1".to_string()),
                    NixAst::String("dep2".to_string()),
                    NixAst::Integer(42),
                ]),
            )
            .build();

        // Find all strings
        let strings = AstManipulator::find_nodes(&ast, &|node| matches!(node, NixAst::String(_)));

        assert_eq!(strings.len(), 3); // "test", "dep1", "dep2"

        // Find all integers
        let integers = AstManipulator::find_nodes(&ast, &|node| matches!(node, NixAst::Integer(_)));

        assert_eq!(integers.len(), 1); // 42
    }

    /// Test AST transformation
    ///
    /// ```mermaid
    /// graph TD
    ///     A[Original AST] --> B[Transform Function]
    ///     B --> C{Match Node?}
    ///     C -->|Yes| D[Replace Node]
    ///     C -->|No| E[Keep Original]
    ///     D --> F[New AST]
    ///     E --> F
    /// ```
    #[test]
    fn test_transform_nodes() {
        let mut ast = AstBuilder::new()
            .attr_set()
            .add_attr("old_name", NixAst::String("old_value".to_string()))
            .add_attr("keep_this", NixAst::String("unchanged".to_string()))
            .build();

        // Transform all strings containing "old"
        AstManipulator::transform_nodes(&mut ast, &|node| match node {
            NixAst::String(s) if s.contains("old") => Some(NixAst::String(s.replace("old", "new"))),
            _ => None,
        });

        // Verify transformation
        match &ast {
            NixAst::AttrSet { bindings, .. } => {
                let first_value = match &bindings[0].value {
                    BindingValue::Value(v) => v,
                    _ => panic!("Expected value"),
                };

                match first_value {
                    NixAst::String(s) => assert_eq!(s, "new_value"),
                    _ => panic!("Expected string"),
                }

                let second_value = match &bindings[1].value {
                    BindingValue::Value(v) => v,
                    _ => panic!("Expected value"),
                };

                match second_value {
                    NixAst::String(s) => assert_eq!(s, "unchanged"),
                    _ => panic!("Expected string"),
                }
            }
            _ => panic!("Expected attribute set"),
        }
    }

    /// Test complex AST structures
    ///
    /// ```mermaid
    /// graph TD
    ///     A[Complex Structure] --> B[Functions]
    ///     A --> C[Let Bindings]
    ///     A --> D[If Expressions]
    ///     B --> E[Parameters]
    ///     B --> F[Body]
    ///     C --> G[Bindings]
    ///     C --> H[Body]
    /// ```
    #[test]
    fn test_complex_ast_structures() {
        // Test function creation
        let func = AstBuilder::simple_function(
            "x",
            AstBuilder::if_expr(
                AstBuilder::binary_op(
                    BinaryOperator::Greater,
                    NixAst::Identifier("x".to_string()),
                    NixAst::Integer(0),
                ),
                NixAst::String("positive".to_string()),
                NixAst::String("non-positive".to_string()),
            ),
        );

        match &func {
            NixAst::Function { param, body } => {
                match param {
                    FunctionParam::Identifier(name) => assert_eq!(name, "x"),
                    _ => panic!("Expected identifier parameter"),
                }

                match body.as_ref() {
                    NixAst::If {
                        condition,
                        then_branch,
                        else_branch,
                    } => {
                        // Verify condition is a binary op
                        matches!(condition.as_ref(), NixAst::BinaryOp { .. });

                        // Verify branches are strings
                        matches!(then_branch.as_ref(), NixAst::String(_));
                        matches!(else_branch.as_ref(), NixAst::String(_));
                    }
                    _ => panic!("Expected if expression"),
                }
            }
            _ => panic!("Expected function"),
        }
    }

    /// Test error handling
    ///
    /// ```mermaid
    /// graph TD
    ///     A[Invalid Input] --> B[Parse Attempt]
    ///     B --> C{Parse Error?}
    ///     C -->|Yes| D[Return Error]
    ///     C -->|No| E[Unexpected Success]
    ///     D --> F[Validate Error Type]
    /// ```
    #[test]
    fn test_parse_errors() {
        let parser = AdvancedParser::new();

        // Test invalid syntax - unclosed attribute set
        let result = parser.parse_string("{ invalid syntax");
        assert!(result.is_err()); // AdvancedParser should detect syntax errors

        // Test empty input
        let result = parser.parse_string("");
        assert!(result.is_err()); // Empty input should be an error

        // Test invalid expression
        let result = parser.parse_string("{ = }");
        assert!(result.is_err()); // Invalid attribute set should be an error
    }

    /// Test attribute path handling
    ///
    /// ```mermaid
    /// graph LR
    ///     A[Attr Path] --> B[Segments]
    ///     B --> C[Identifier]
    ///     B --> D[String]
    ///     B --> E[Interpolation]
    /// ```
    #[test]
    fn test_attribute_paths() {
        let path = AttrPath {
            segments: vec![
                AttrPathSegment::Identifier("foo".to_string()),
                AttrPathSegment::Dynamic(NixAst::String("bar baz".to_string())),
            ],
        };

        assert_eq!(path.segments.len(), 2);

        match &path.segments[0] {
            AttrPathSegment::Identifier(s) => assert_eq!(s, "foo"),
            _ => panic!("Expected identifier"),
        }

        match &path.segments[1] {
            AttrPathSegment::Dynamic(NixAst::String(s)) => assert_eq!(s, "bar baz"),
            _ => panic!("Expected dynamic string"),
        }
    }

    /// Test location tracking
    ///
    /// ```mermaid
    /// graph TD
    ///     A[Parse with Location] --> B[Track Line/Column]
    ///     B --> C[Track Offset]
    ///     C --> D[Track Length]
    ///     D --> E[Verify Location Data]
    /// ```
    #[test]
    fn test_location_tracking() {
        let location = Location {
            file: Some(PathBuf::from("test.nix")),
            line: 5,
            column: 10,
            offset: 150,
            length: 25,
        };

        assert_eq!(location.line, 5);
        assert_eq!(location.column, 10);
        assert_eq!(location.offset, 150);
        assert_eq!(location.length, 25);
        assert_eq!(
            location.file.as_ref().unwrap().to_str().unwrap(),
            "test.nix"
        );
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;

    /// Test full parsing and manipulation workflow
    ///
    /// ```mermaid
    /// graph TD
    ///     A[Nix Source] --> B[Parse]
    ///     B --> C[Manipulate]
    ///     C --> D[Query]
    ///     D --> E[Transform]
    ///     E --> F[Validate Result]
    /// ```
    #[test]
    fn test_full_workflow() {
        let parser = AdvancedParser::new();

        // Parse a simple expression
        let parsed = parser.parse_string(r#"{ x = 1; y = 2; }"#).unwrap();

        // AdvancedParser returns a proper AttrSet
        match &parsed.ast {
            NixAst::AttrSet { bindings, .. } => {
                assert_eq!(bindings.len(), 2);

                // Find the x binding
                let x_binding = bindings
                    .iter()
                    .find(|b| {
                        matches!(
                            b.attr_path.segments.first(),
                            Some(AttrPathSegment::Identifier(s)) if s == "x"
                        )
                    })
                    .expect("x binding not found");

                // Verify x = 1
                match &x_binding.value {
                    BindingValue::Value(NixAst::Integer(1)) => {}
                    _ => panic!("Expected x = 1"),
                }

                // Find the y binding
                let y_binding = bindings
                    .iter()
                    .find(|b| {
                        matches!(
                            b.attr_path.segments.first(),
                            Some(AttrPathSegment::Identifier(s)) if s == "y"
                        )
                    })
                    .expect("y binding not found");

                // Verify y = 2
                match &y_binding.value {
                    BindingValue::Value(NixAst::Integer(2)) => {}
                    _ => panic!("Expected y = 2"),
                }
            }
            _ => panic!("Expected AttrSet, got {:?}", parsed.ast),
        }
    }
}

#[test]
fn test_parse_literals() {
    // Integer
    let int_file = NixFile::parse_string("42".to_string(), None).unwrap();
    let ast = int_file.to_ast().unwrap();
    match ast {
        NixAst::Integer(42) => {}
        _ => panic!("Expected Integer(42), got {:?}", ast),
    }

    // Float
    let float_file = NixFile::parse_string("3.14".to_string(), None).unwrap();
    let ast = float_file.to_ast().unwrap();
    match ast {
        NixAst::Float(f) => assert!((f - 3.14).abs() < 0.001),
        _ => panic!("Expected Float(3.14), got {:?}", ast),
    }

    // String
    let string_file = NixFile::parse_string(r#""hello world""#.to_string(), None).unwrap();
    let ast = string_file.to_ast().unwrap();
    match ast {
        NixAst::String(s) => assert_eq!(s, "hello world"),
        _ => panic!("Expected String, got {:?}", ast),
    }

    // Boolean
    let bool_file = NixFile::parse_string("true".to_string(), None).unwrap();
    let ast = bool_file.to_ast().unwrap();
    match ast {
        NixAst::Bool(true) => {}
        _ => panic!("Expected Bool(true), got {:?}", ast),
    }

    // Null
    let null_file = NixFile::parse_string("null".to_string(), None).unwrap();
    let ast = null_file.to_ast().unwrap();
    match ast {
        NixAst::Null => {}
        _ => panic!("Expected Null, got {:?}", ast),
    }
}

#[test]
fn test_parse_list() {
    let list_file = NixFile::parse_string("[1 2 3]".to_string(), None).unwrap();
    let ast = list_file.to_ast().unwrap();
    match ast {
        NixAst::List(items) => {
            assert_eq!(items.len(), 3);
            match &items[0] {
                NixAst::Integer(1) => {}
                _ => panic!("Expected Integer(1)"),
            }
            match &items[1] {
                NixAst::Integer(2) => {}
                _ => panic!("Expected Integer(2)"),
            }
            match &items[2] {
                NixAst::Integer(3) => {}
                _ => panic!("Expected Integer(3)"),
            }
        }
        _ => panic!("Expected List, got {:?}", ast),
    }
}

#[test]
fn test_parse_attrset() {
    let attrset_file =
        NixFile::parse_string(r#"{ foo = 42; bar = "hello"; }"#.to_string(), None).unwrap();
    let ast = attrset_file.to_ast().unwrap();
    match ast {
        NixAst::AttrSet {
            recursive,
            bindings,
        } => {
            assert!(!recursive);
            assert_eq!(bindings.len(), 2);

            // Check first binding
            let binding = &bindings[0];
            assert_eq!(binding.attr_path.segments.len(), 1);
            match &binding.attr_path.segments[0] {
                AttrPathSegment::Identifier(name) => assert_eq!(name, "foo"),
                _ => panic!("Expected identifier"),
            }
            match &binding.value {
                BindingValue::Value(NixAst::Integer(42)) => {}
                _ => panic!("Expected Value(Integer(42))"),
            }
        }
        _ => panic!("Expected AttrSet, got {:?}", ast),
    }
}

#[test]
fn test_parse_recursive_attrset() {
    let rec_file = NixFile::parse_string(r#"rec { a = 1; b = a + 1; }"#.to_string(), None).unwrap();
    let ast = rec_file.to_ast().unwrap();
    match ast {
        NixAst::AttrSet { recursive, .. } => {
            assert!(recursive);
        }
        _ => panic!("Expected AttrSet, got {:?}", ast),
    }
}

#[test]
fn test_parse_function() {
    // Simple function
    let fn_file = NixFile::parse_string("x: x + 1".to_string(), None).unwrap();
    let ast = fn_file.to_ast().unwrap();
    match ast {
        NixAst::Function { param, body } => {
            match param {
                FunctionParam::Identifier(name) => assert_eq!(name, "x"),
                _ => panic!("Expected identifier parameter"),
            }
            match body.as_ref() {
                NixAst::BinaryOp {
                    op: BinaryOperator::Add,
                    ..
                } => {}
                _ => panic!("Expected binary addition"),
            }
        }
        _ => panic!("Expected Function, got {:?}", ast),
    }

    // Pattern function
    let pattern_file = NixFile::parse_string("{ a, b }: a + b".to_string(), None).unwrap();
    let ast = pattern_file.to_ast().unwrap();
    match ast {
        NixAst::Function { param, .. } => match param {
            FunctionParam::Pattern {
                fields, ellipsis, ..
            } => {
                assert_eq!(fields.len(), 2);
                assert!(!ellipsis);
            }
            _ => panic!("Expected pattern parameter"),
        },
        _ => panic!("Expected Function, got {:?}", ast),
    }
}

#[test]
fn test_parse_let() {
    let let_file = NixFile::parse_string("let x = 1; y = 2; in x + y".to_string(), None).unwrap();
    let ast = let_file.to_ast().unwrap();
    match ast {
        NixAst::Let { bindings, body } => {
            assert_eq!(bindings.len(), 2);
            match body.as_ref() {
                NixAst::BinaryOp {
                    op: BinaryOperator::Add,
                    ..
                } => {}
                _ => panic!("Expected binary addition in body"),
            }
        }
        _ => panic!("Expected Let, got {:?}", ast),
    }
}

#[test]
fn test_parse_if() {
    let if_file = NixFile::parse_string("if true then 1 else 2".to_string(), None).unwrap();
    let ast = if_file.to_ast().unwrap();
    match ast {
        NixAst::If {
            condition,
            then_branch,
            else_branch,
        } => {
            match condition.as_ref() {
                NixAst::Bool(true) => {}
                _ => panic!("Expected Bool(true) condition"),
            }
            match then_branch.as_ref() {
                NixAst::Integer(1) => {}
                _ => panic!("Expected Integer(1) in then branch"),
            }
            match else_branch.as_ref() {
                NixAst::Integer(2) => {}
                _ => panic!("Expected Integer(2) in else branch"),
            }
        }
        _ => panic!("Expected If, got {:?}", ast),
    }
}

#[test]
fn test_parse_binary_ops() {
    let tests = vec![
        ("1 + 2", BinaryOperator::Add),
        ("1 - 2", BinaryOperator::Subtract),
        ("1 * 2", BinaryOperator::Multiply),
        ("1 / 2", BinaryOperator::Divide),
        ("1 == 2", BinaryOperator::Equal),
        ("1 != 2", BinaryOperator::NotEqual),
        ("1 < 2", BinaryOperator::Less),
        ("1 <= 2", BinaryOperator::LessEqual),
        ("1 > 2", BinaryOperator::Greater),
        ("1 >= 2", BinaryOperator::GreaterEqual),
        ("true && false", BinaryOperator::And),
        ("true || false", BinaryOperator::Or),
        ("[1] ++ [2]", BinaryOperator::Concat),
        ("{ a = 1; } // { b = 2; }", BinaryOperator::Update),
    ];

    for (expr, expected_op) in tests {
        let file = NixFile::parse_string(expr.to_string(), None).unwrap();
        let ast = file.to_ast().unwrap();
        match ast {
            NixAst::BinaryOp { op, .. } => {
                assert_eq!(op, expected_op, "Failed for expression: {}", expr);
            }
            _ => panic!("Expected BinaryOp for {}, got {:?}", expr, ast),
        }
    }
}

#[test]
fn test_parse_unary_ops() {
    // Negation
    let neg_file = NixFile::parse_string("-5".to_string(), None).unwrap();
    let ast = neg_file.to_ast().unwrap();
    match ast {
        NixAst::UnaryOp {
            op: UnaryOperator::Negate,
            operand,
        } => match operand.as_ref() {
            NixAst::Integer(5) => {}
            _ => panic!("Expected Integer(5) operand"),
        },
        _ => panic!("Expected UnaryOp, got {:?}", ast),
    }

    // Not
    let not_file = NixFile::parse_string("!true".to_string(), None).unwrap();
    let ast = not_file.to_ast().unwrap();
    match ast {
        NixAst::UnaryOp {
            op: UnaryOperator::Not,
            operand,
        } => match operand.as_ref() {
            NixAst::Bool(true) => {}
            _ => panic!("Expected Bool(true) operand"),
        },
        _ => panic!("Expected UnaryOp, got {:?}", ast),
    }
}

#[test]
fn test_parse_select() {
    let select_file = NixFile::parse_string("foo.bar".to_string(), None).unwrap();
    let ast = select_file.to_ast().unwrap();
    match ast {
        NixAst::Select {
            expr,
            attr_path,
            default,
        } => {
            match expr.as_ref() {
                NixAst::Identifier(name) => assert_eq!(name, "foo"),
                _ => panic!("Expected Identifier"),
            }
            assert_eq!(attr_path.segments.len(), 1);
            assert!(default.is_none());
        }
        _ => panic!("Expected Select, got {:?}", ast),
    }

    // With default
    let select_default_file = NixFile::parse_string("foo.bar or 42".to_string(), None).unwrap();
    let ast = select_default_file.to_ast().unwrap();
    match ast {
        NixAst::Select { default, .. } => {
            assert!(default.is_some());
            match default.unwrap().as_ref() {
                NixAst::Integer(42) => {}
                _ => panic!("Expected Integer(42) as default"),
            }
        }
        _ => panic!("Expected Select, got {:?}", ast),
    }
}

#[test]
fn test_parse_has_attr() {
    let has_attr_file = NixFile::parse_string("foo ? bar".to_string(), None).unwrap();
    let ast = has_attr_file.to_ast().unwrap();
    match ast {
        NixAst::HasAttr { expr, attr_path } => {
            match expr.as_ref() {
                NixAst::Identifier(name) => assert_eq!(name, "foo"),
                _ => panic!("Expected Identifier"),
            }
            assert_eq!(attr_path.segments.len(), 1);
        }
        _ => panic!("Expected HasAttr, got {:?}", ast),
    }
}

#[test]
fn test_parse_with() {
    let with_file = NixFile::parse_string("with foo; bar".to_string(), None).unwrap();
    let ast = with_file.to_ast().unwrap();
    match ast {
        NixAst::With { namespace, body } => {
            match namespace.as_ref() {
                NixAst::Identifier(name) => assert_eq!(name, "foo"),
                _ => panic!("Expected Identifier namespace"),
            }
            match body.as_ref() {
                NixAst::Identifier(name) => assert_eq!(name, "bar"),
                _ => panic!("Expected Identifier body"),
            }
        }
        _ => panic!("Expected With, got {:?}", ast),
    }
}

#[test]
fn test_parse_assert() {
    let assert_file = NixFile::parse_string("assert true; 42".to_string(), None).unwrap();
    let ast = assert_file.to_ast().unwrap();
    match ast {
        NixAst::Assert { condition, body } => {
            match condition.as_ref() {
                NixAst::Bool(true) => {}
                _ => panic!("Expected Bool(true) condition"),
            }
            match body.as_ref() {
                NixAst::Integer(42) => {}
                _ => panic!("Expected Integer(42) body"),
            }
        }
        _ => panic!("Expected Assert, got {:?}", ast),
    }
}

#[test]
fn test_parse_import() {
    let import_file = NixFile::parse_string("import ./foo.nix".to_string(), None).unwrap();
    let ast = import_file.to_ast().unwrap();
    match ast {
        NixAst::Import(expr) => match expr.as_ref() {
            NixAst::Path(_) => {}
            _ => panic!("Expected Path in import"),
        },
        _ => panic!("Expected Import, got {:?}", ast),
    }
}

#[test]
fn test_parse_inherit() {
    let inherit_file = NixFile::parse_string("{ inherit foo bar; }".to_string(), None).unwrap();
    let ast = inherit_file.to_ast().unwrap();
    match ast {
        NixAst::AttrSet { bindings, .. } => {
            assert_eq!(bindings.len(), 2);
            for binding in bindings {
                match &binding.value {
                    BindingValue::Inherit { from, attrs } => {
                        assert!(from.is_none());
                        assert_eq!(attrs.len(), 1);
                    }
                    _ => panic!("Expected Inherit binding"),
                }
            }
        }
        _ => panic!("Expected AttrSet, got {:?}", ast),
    }

    // Inherit from
    let inherit_from_file =
        NixFile::parse_string("{ inherit (foo) bar baz; }".to_string(), None).unwrap();
    let ast = inherit_from_file.to_ast().unwrap();
    match ast {
        NixAst::AttrSet { bindings, .. } => {
            assert_eq!(bindings.len(), 2);
            for binding in bindings {
                match &binding.value {
                    BindingValue::Inherit { from, .. } => {
                        assert!(from.is_some());
                    }
                    _ => panic!("Expected Inherit binding"),
                }
            }
        }
        _ => panic!("Expected AttrSet, got {:?}", ast),
    }
}

#[test]
fn test_parse_complex_expression() {
    let complex = r#"
    let
      x = 1;
      y = 2;
      f = a: b: a + b;
    in
      f x y
    "#;

    let file = NixFile::parse_string(complex.to_string(), None).unwrap();
    let ast = file.to_ast().unwrap();
    match ast {
        NixAst::Let { bindings, body } => {
            assert_eq!(bindings.len(), 3);
            // Check that body is a function application
            match body.as_ref() {
                NixAst::Apply { .. } => {}
                _ => panic!("Expected Apply in body"),
            }
        }
        _ => panic!("Expected Let, got {:?}", ast),
    }
}

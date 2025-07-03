//! Demonstrates the AST manipulation capabilities of the Nix domain

use cim_domain_nix::parser::{AstManipulator, AstBuilder, NixAst};
use cim_domain_nix::parser::ast::{Binding, BindingValue, AttrPath, AttrPathSegment, BinaryOperator};

fn main() {
    println!("=== Nix AST Manipulation Demo ===\n");

    // Example 1: Build an attribute set from scratch
    println!("1. Building an attribute set:");
    let mut ast = AstBuilder::new()
        .attr_set()
        .add_attr("name", NixAst::String("hello".to_string()))
        .add_attr("version", NixAst::String("2.12".to_string()))
        .add_attr("enabled", NixAst::Bool(true))
        .build();
    
    println!("Built AST: {:#?}\n", ast);

    // Example 2: Add more attributes
    println!("2. Adding attributes:");
    AstManipulator::add_attribute(
        &mut ast,
        vec!["meta", "description"],
        NixAst::String("A friendly greeting program".to_string()),
    ).unwrap();
    
    AstManipulator::add_attribute(
        &mut ast,
        vec!["meta", "license"],
        NixAst::String("MIT".to_string()),
    ).unwrap();
    
    println!("After adding meta attributes: {:#?}\n", ast);

    // Example 3: Query attributes
    println!("3. Querying attributes:");
    if let Some(version) = AstManipulator::get_attribute(&ast, vec!["version"]) {
        println!("Version: {:?}", version);
    }
    
    if let Some(desc) = AstManipulator::get_attribute(&ast, vec!["meta", "description"]) {
        println!("Description: {:?}", desc);
    }
    println!();

    // Example 4: Transform nodes
    println!("4. Transforming nodes:");
    let mut numbers = AstBuilder::list(vec![
        NixAst::Integer(1),
        NixAst::Integer(2),
        NixAst::Integer(3),
        NixAst::Integer(4),
        NixAst::Integer(5),
    ]);
    
    println!("Original list: {:?}", numbers);
    
    AstManipulator::transform_nodes(&mut numbers, &|node| {
        match node {
            NixAst::Integer(n) => Some(NixAst::Integer(*n * 10)),
            _ => None,
        }
    });
    
    println!("After multiplying by 10: {:?}\n", numbers);

    // Example 5: Build complex expressions
    println!("5. Building complex expressions:");
    let condition = AstBuilder::binary_op(
        BinaryOperator::Greater,
        NixAst::Identifier("x".to_string()),
        NixAst::Integer(0),
    );
    
    let if_expr = AstBuilder::if_expr(
        condition,
        NixAst::String("positive".to_string()),
        NixAst::String("non-positive".to_string()),
    );
    
    let func = AstBuilder::simple_function("x", if_expr);
    
    println!("Function AST: {:#?}\n", func);

    // Example 6: Find specific nodes
    println!("6. Finding nodes:");
    let complex_ast = NixAst::AttrSet {
        recursive: false,
        bindings: vec![
            Binding {
                attr_path: AttrPath {
                    segments: vec![AttrPathSegment::Identifier("numbers".to_string())],
                },
                value: BindingValue::Value(NixAst::List(vec![
                    NixAst::Integer(1),
                    NixAst::Integer(2),
                    NixAst::Integer(3),
                ])),
            },
            Binding {
                attr_path: AttrPath {
                    segments: vec![AttrPathSegment::Identifier("strings".to_string())],
                },
                value: BindingValue::Value(NixAst::List(vec![
                    NixAst::String("hello".to_string()),
                    NixAst::String("world".to_string()),
                ])),
            },
        ],
    };
    
    let integers = AstManipulator::find_nodes(&complex_ast, &|node| {
        matches!(node, NixAst::Integer(_))
    });
    
    println!("Found {integers.len(} integer nodes"));
    
    let strings = AstManipulator::find_nodes(&complex_ast, &|node| {
        matches!(node, NixAst::String(_))
    });
    
    println!("Found {strings.len(} string nodes"));
    
    println!("\n=== Demo Complete ===");
} 
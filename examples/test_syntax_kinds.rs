use rnix::{parser, tokenizer};

fn main() {
    // Test attribute set
    let code1 = r#"{ foo = "bar"; }"#;
    println!("Testing attribute set: {code1}");
    test_code(code1);

    // Test let expression
    let code2 = r#"let x = 42; y = 99; in x"#;
    println!("\nTesting let expression: {code2}");
    test_code(code2);
}

fn test_code(code: &str) {
    // Tokenize first
    let tokens = tokenizer::tokenize(code);

    // Then parse - tokens is already an iterator
    let (green, errors) = parser::parse(tokens.into_iter());

    println!("Parse errors: {:?}", errors);

    // Create syntax node from green node
    let root = rnix::SyntaxNode::new_root(green);

    print_tree(&root, 0);
}

fn print_tree(node: &rnix::SyntaxNode, depth: usize) {
    let indent = "  ".repeat(depth);
    println!(
        "{}{:?} - \"{}\"",
        indent,
        node.kind(),
        node.text().to_string().replace('\n', "\\n")
    );

    for child in node.children() {
        print_tree(&child, depth + 1);
    }
}

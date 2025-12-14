// Example to inspect AST structure

use rnix::SyntaxKind;

fn print_tree(node: &rnix::SyntaxNode, depth: usize) {
    let indent = "  ".repeat(depth);
    println!("{}{:?}: '{}'", indent, node.kind(), node.text());

    for child in node.children() {
        print_tree(&child, depth + 1);
    }
}

fn main() {
    let source = r#"{
        name = "test";
        value = 42;
    }"#;

    let parse = rnix::Root::parse(source);

    println!("=== AST Structure ===");
    print_tree(&parse.syntax(), 0);

    println!("\n=== Available SyntaxKind variants (sample) ===");
    println!("ATTR_SET: {:?}", SyntaxKind::NODE_ATTR_SET);
    println!("LIST: {:?}", SyntaxKind::NODE_LIST);
    println!("STRING: {:?}", SyntaxKind::NODE_STRING);
}

use rnix::{SyntaxKind, tokenizer, parser};

fn main() {
    let code = r#"{ foo = "bar"; }"#;
    
    // Tokenize first
    let tokens = tokenizer::tokenize(code);
    
    // Then parse
    let (green, errors) = parser::parse(tokens);
    
    println!("Parse errors: {:?}", errors);
    
    // Create syntax node from green node
    let root = rnix::SyntaxNode::new_root(green);
    
    println!("Root kind: {:?}", root.kind());
    
    for child in root.children() {
        println!("  Child kind: {:?}", child.kind());
        if child.kind() == SyntaxKind::NODE_ATTR_SET {
            for grandchild in child.children() {
                println!("    Grandchild kind: {:?}", grandchild.kind());
            }
        }
    }
} 
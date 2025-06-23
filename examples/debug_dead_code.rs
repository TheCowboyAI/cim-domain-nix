use cim_domain_nix::parser::NixFile;
use rnix::{SyntaxKind, SyntaxNode};

fn print_ast(node: &SyntaxNode, indent: usize) {
    let prefix = " ".repeat(indent);
    println!("{}[{:?}] '{}'", prefix, node.kind(), node.text().to_string().trim());
    
    for child in node.children() {
        print_ast(&child, indent + 2);
    }
}

fn main() {
    let content = r#"
    let
      used = 42;
      unused = 99;
    in used
    "#;

    let file = NixFile::parse_string(content.to_string(), None).unwrap();
    
    println!("=== AST Structure ===");
    print_ast(&file.ast, 0);
    
    println!("\n=== Looking for let bindings ===");
    find_let_bindings(&file.ast);
}

fn find_let_bindings(node: &SyntaxNode) {
    if node.kind() == SyntaxKind::NODE_LET_IN {
        println!("Found NODE_LET_IN");
        
        // Look for bindings
        for child in node.children() {
            println!("  Child: {:?}", child.kind());
            
            if child.kind() == SyntaxKind::NODE_ATTR_SET {
                println!("    Found NODE_ATTR_SET");
                
                // Look for bindings in the attr set
                for attr_child in child.children() {
                    println!("      Attr child: {:?} = '{}'", attr_child.kind(), attr_child.text().to_string().trim());
                    
                    if attr_child.kind() == SyntaxKind::NODE_ATTRPATH_VALUE {
                        println!("        Found NODE_ATTRPATH_VALUE");
                        
                        // Look for the identifier
                        for value_child in attr_child.children() {
                            println!("          Value child: {:?} = '{}'", value_child.kind(), value_child.text());
                            
                            if value_child.kind() == SyntaxKind::NODE_ATTRPATH {
                                // Look inside the attrpath
                                for path_child in value_child.children() {
                                    println!("            Path child: {:?} = '{}'", path_child.kind(), path_child.text());
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    
    // Recurse
    for child in node.children() {
        find_let_bindings(&child);
    }
} 
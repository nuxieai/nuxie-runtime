use crate::functions::to_string_pretty_printer::to_string_ast_node;
use crate::records::ast_node::AstNode;

pub fn dump(node: *mut AstNode) {
    let s = to_string_ast_node(node);
    println!("{}", s);
}

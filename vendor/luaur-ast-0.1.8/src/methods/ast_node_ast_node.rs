use crate::records::ast_node::AstNode;
use crate::records::location::Location;

impl AstNode {
    pub fn new(class_index: i32, location: Location) -> Self {
        Self {
            class_index,
            location,
        }
    }
}

#[allow(non_snake_case)]
pub fn ast_node_ast_node(class_index: i32, location: Location) -> AstNode {
    AstNode::new(class_index, location)
}

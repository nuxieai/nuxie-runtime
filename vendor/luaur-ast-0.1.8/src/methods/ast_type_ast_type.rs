use crate::records::ast_node::AstNode;
use crate::records::ast_type::AstType;
use crate::records::location::Location;

impl AstType {
    pub fn new(class_index: i32, location: Location) -> Self {
        Self {
            base: AstNode {
                class_index,
                location,
            },
        }
    }
}

#[allow(non_snake_case)]
pub fn ast_type_ast_type(class_index: i32, location: Location) -> AstType {
    AstType::new(class_index, location)
}

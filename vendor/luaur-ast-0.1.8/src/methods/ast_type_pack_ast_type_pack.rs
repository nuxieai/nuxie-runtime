use crate::records::ast_node::AstNode;
use crate::records::ast_type_pack::AstTypePack;
use crate::records::location::Location;

impl AstTypePack {
    pub fn new(class_index: i32, location: Location) -> Self {
        Self {
            base: AstNode {
                class_index,
                location,
            },
        }
    }
}

use crate::records::ast_node::AstNode;
use crate::records::ast_type::AstType;
use crate::records::ast_type_group::AstTypeGroup;
use crate::records::location::Location;
use crate::rtti::AstNodeClass;

impl AstTypeGroup {
    pub fn new(location: Location, type_: *mut AstType) -> Self {
        Self {
            base: AstType {
                base: AstNode {
                    class_index: <Self as AstNodeClass>::CLASS_INDEX,
                    location,
                },
            },
            type_,
        }
    }
}

#[allow(non_snake_case)]
pub fn ast_type_group_ast_type_group(location: Location, type_: *mut AstType) -> AstTypeGroup {
    AstTypeGroup::new(location, type_)
}

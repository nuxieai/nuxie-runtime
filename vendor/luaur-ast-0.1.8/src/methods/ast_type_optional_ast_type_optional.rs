use crate::records::ast_node::AstNode;
use crate::records::ast_type::AstType;
use crate::records::ast_type_optional::AstTypeOptional;
use crate::records::location::Location;
use crate::rtti::AstNodeClass;

impl AstTypeOptional {
    pub fn new(location: Location) -> Self {
        Self {
            base: AstType {
                base: AstNode {
                    class_index: <Self as AstNodeClass>::CLASS_INDEX,
                    location,
                },
            },
            type_: core::ptr::null_mut(),
        }
    }
}

#[allow(non_snake_case)]
pub fn ast_type_optional_ast_type_optional(location: Location) -> AstTypeOptional {
    AstTypeOptional::new(location)
}

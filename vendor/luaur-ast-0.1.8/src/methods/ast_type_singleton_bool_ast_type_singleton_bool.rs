use crate::records::ast_node::AstNode;
use crate::records::ast_type::AstType;
use crate::records::ast_type_singleton_bool::AstTypeSingletonBool;
use crate::records::location::Location;
use crate::rtti::AstNodeClass;

impl AstTypeSingletonBool {
    pub fn new(location: Location, value: bool) -> Self {
        AstTypeSingletonBool {
            base: AstType {
                base: AstNode {
                    class_index: <AstTypeSingletonBool as crate::rtti::AstNodeClass>::CLASS_INDEX,
                    location,
                },
            },
            value,
        }
    }
}

#[allow(non_snake_case)]
pub fn ast_type_singleton_bool_ast_type_singleton_bool(
    location: Location,
    value: bool,
) -> AstTypeSingletonBool {
    AstTypeSingletonBool::new(location, value)
}

use crate::records::ast_node::AstNode;
use crate::records::ast_type::AstType;
use crate::records::ast_type_pack::AstTypePack;
use crate::records::ast_type_pack_variadic::AstTypePackVariadic;
use crate::records::location::Location;
use crate::rtti::AstNodeClass;

impl AstTypePackVariadic {
    pub fn new(location: Location, variadic_type: *mut AstType) -> Self {
        Self {
            base: AstTypePack {
                base: AstNode {
                    class_index: <Self as AstNodeClass>::CLASS_INDEX,
                    location,
                },
            },
            variadic_type: variadic_type,
        }
    }
}

#[allow(non_snake_case)]
pub fn ast_type_pack_variadic_ast_type_pack_variadic(
    location: Location,
    variadic_type: *mut AstType,
) -> AstTypePackVariadic {
    AstTypePackVariadic::new(location, variadic_type)
}

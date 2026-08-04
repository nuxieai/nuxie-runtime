use crate::records::ast_generic_type::AstGenericType;
use crate::records::ast_name::AstName;
use crate::records::ast_node::AstNode;
use crate::records::ast_type::AstType;
use crate::records::location::Location;
use crate::rtti::AstNodeClass;

impl AstGenericType {
    pub fn new(location: Location, name: AstName, default_value: *mut AstType) -> Self {
        Self {
            base: AstNode {
                class_index: <Self as AstNodeClass>::CLASS_INDEX,
                location,
            },
            name,
            default_value,
        }
    }
}

#[allow(non_snake_case)]
pub fn ast_generic_type_ast_generic_type(
    location: Location,
    name: AstName,
    default_value: *mut AstType,
) -> AstGenericType {
    AstGenericType::new(location, name, default_value)
}

use crate::records::ast_array::AstArray;
use crate::records::ast_node::AstNode;
use crate::records::ast_type::AstType;
use crate::records::ast_type_singleton_string::AstTypeSingletonString;
use crate::records::location::Location;
use crate::rtti::AstNodeClass;

impl AstTypeSingletonString {
    pub fn new(location: Location, value: AstArray<core::ffi::c_char>) -> Self {
        Self {
            base: AstType {
                base: AstNode {
                    class_index: <Self as AstNodeClass>::CLASS_INDEX,
                    location,
                },
            },
            value,
        }
    }
}

#[allow(non_snake_case)]
pub fn ast_type_singleton_string_ast_type_singleton_string(
    location: Location,
    value: AstArray<core::ffi::c_char>,
) -> AstTypeSingletonString {
    AstTypeSingletonString::new(location, value)
}

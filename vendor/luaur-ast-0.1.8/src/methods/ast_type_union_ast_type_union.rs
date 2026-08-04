use crate::records::ast_array::AstArray;
use crate::records::ast_node::AstNode;
use crate::records::ast_type::AstType;
use crate::records::ast_type_union::AstTypeUnion;
use crate::records::location::Location;
use crate::rtti::AstNodeClass;

impl AstTypeUnion {
    pub fn new(location: Location, types: AstArray<*mut AstType>) -> Self {
        Self {
            base: AstType {
                base: AstNode {
                    class_index: <Self as AstNodeClass>::CLASS_INDEX,
                    location,
                },
            },
            types,
        }
    }
}

#[allow(non_snake_case)]
pub fn ast_type_union_ast_type_union(
    location: Location,
    types: AstArray<*mut AstType>,
) -> AstTypeUnion {
    AstTypeUnion::new(location, types)
}

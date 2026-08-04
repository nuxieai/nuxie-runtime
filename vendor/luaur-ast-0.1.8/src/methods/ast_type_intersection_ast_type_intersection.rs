use crate::records::ast_array::AstArray;
use crate::records::ast_node::AstNode;
use crate::records::ast_type::AstType;
use crate::records::ast_type_intersection::AstTypeIntersection;
use crate::records::location::Location;
use crate::rtti::AstNodeClass;

impl AstTypeIntersection {
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
pub fn ast_type_intersection_ast_type_intersection(
    location: Location,
    types: AstArray<*mut AstType>,
) -> AstTypeIntersection {
    AstTypeIntersection::new(location, types)
}

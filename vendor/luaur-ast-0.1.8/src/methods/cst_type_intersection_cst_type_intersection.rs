use crate::records::ast_array::AstArray;
use crate::records::cst_node::CstNode;
use crate::records::cst_type_intersection::CstTypeIntersection;
use crate::records::position::Position;
use crate::rtti::CstNodeClass;

impl CstTypeIntersection {
    pub fn new(leading_position: Position, separator_positions: AstArray<Position>) -> Self {
        Self {
            base: CstNode {
                class_index: <Self as CstNodeClass>::CLASS_INDEX,
            },
            leading_position,
            separator_positions,
        }
    }
}

#[allow(non_snake_case)]
pub fn cst_type_intersection_cst_type_intersection(
    leading_position: Position,
    separator_positions: AstArray<Position>,
) -> CstTypeIntersection {
    CstTypeIntersection::new(leading_position, separator_positions)
}

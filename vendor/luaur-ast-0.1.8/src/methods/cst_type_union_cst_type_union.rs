use crate::records::ast_array::AstArray;
use crate::records::cst_node::CstNode;
use crate::records::cst_type_union::CstTypeUnion;
use crate::records::position::Position;
use crate::rtti::CstNodeClass;

impl CstTypeUnion {
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
pub fn cst_type_union_cst_type_union(
    leading_position: Position,
    separator_positions: AstArray<Position>,
) -> CstTypeUnion {
    CstTypeUnion::new(leading_position, separator_positions)
}

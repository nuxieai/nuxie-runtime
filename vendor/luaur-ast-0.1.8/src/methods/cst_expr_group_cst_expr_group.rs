use crate::records::cst_expr_group::CstExprGroup;
use crate::records::position::Position;

impl CstExprGroup {
    pub fn new(close_position: Position) -> Self {
        Self {
            base: crate::records::cst_node::CstNode {
                class_index: <Self as crate::rtti::CstNodeClass>::CLASS_INDEX,
            },
            close_position,
        }
    }
}

#[allow(non_snake_case)]
pub fn cst_expr_group_cst_expr_group(close_position: Position) -> CstExprGroup {
    CstExprGroup::new(close_position)
}

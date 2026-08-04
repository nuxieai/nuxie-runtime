use crate::records::cst_node::CstNode;
use crate::records::cst_stat_compound_assign::CstStatCompoundAssign;
use crate::records::position::Position;
use crate::rtti::CstNodeClass;

impl CstStatCompoundAssign {
    pub fn new(op_position: Position) -> Self {
        Self {
            base: CstNode {
                class_index: <Self as CstNodeClass>::CLASS_INDEX,
            },
            op_position,
        }
    }
}

#[allow(non_snake_case)]
pub fn cst_stat_compound_assign_cst_stat_compound_assign(
    op_position: Position,
) -> CstStatCompoundAssign {
    CstStatCompoundAssign::new(op_position)
}

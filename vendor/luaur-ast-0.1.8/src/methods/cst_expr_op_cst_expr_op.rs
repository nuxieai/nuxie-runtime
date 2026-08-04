use crate::records::cst_expr_op::CstExprOp;
use crate::records::cst_node::CstNode;
use crate::records::position::Position;
use crate::rtti::CstNodeClass;

impl CstExprOp {
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
pub fn cst_expr_op_cst_expr_op(op_position: Position) -> CstExprOp {
    CstExprOp::new(op_position)
}

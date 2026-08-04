use crate::records::cst_node::CstNode;
use crate::records::position::Position;

#[repr(C)]
#[derive(Debug, Clone)]
pub struct CstExprOp {
    pub base: CstNode,
    pub op_position: Position,
}

impl crate::rtti::CstNodeClass for CstExprOp {
    const CLASS_INDEX: i32 = crate::rtti::ast_rtti_index("CstExprOp");
}

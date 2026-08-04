use crate::records::cst_node::CstNode;
use crate::records::position::Position;

#[repr(C)]
#[derive(Debug, Clone)]
pub struct CstExprIfElse {
    pub base: CstNode,
    pub then_position: Position,
    pub else_position: Position,
    pub is_else_if: bool,
}

impl crate::rtti::CstNodeClass for CstExprIfElse {
    const CLASS_INDEX: i32 = crate::rtti::ast_rtti_index("CstExprIfElse");
}

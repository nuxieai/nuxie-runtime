use crate::records::cst_node::CstNode;
use crate::records::position::Position;

#[repr(C)]
#[derive(Debug, Clone)]
pub struct CstExprGroup {
    pub base: CstNode,
    pub close_position: Position,
}

impl crate::rtti::CstNodeClass for CstExprGroup {
    const CLASS_INDEX: i32 = crate::rtti::ast_rtti_index("CstExprGroup");
}

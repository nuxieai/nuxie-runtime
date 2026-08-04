use crate::records::cst_node::CstNode;
use crate::records::position::Position;

#[repr(C)]
#[derive(Debug, Clone)]
pub struct CstStatCompoundAssign {
    pub base: CstNode,
    pub op_position: Position,
}

impl crate::rtti::CstNodeClass for CstStatCompoundAssign {
    const CLASS_INDEX: i32 = crate::rtti::cst_rtti_index("CstStatCompoundAssign");
}

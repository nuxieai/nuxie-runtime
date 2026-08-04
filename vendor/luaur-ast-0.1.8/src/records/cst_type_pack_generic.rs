use crate::records::cst_node::CstNode;
use crate::records::position::Position;

#[repr(C)]
#[derive(Debug, Clone)]
pub struct CstTypePackGeneric {
    pub base: CstNode,
    pub ellipsis_position: Position,
}

impl crate::rtti::CstNodeClass for CstTypePackGeneric {
    const CLASS_INDEX: i32 = crate::rtti::ast_rtti_index("CstTypePackGeneric");
}

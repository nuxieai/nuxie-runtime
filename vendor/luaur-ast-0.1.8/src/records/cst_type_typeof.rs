use crate::records::cst_node::CstNode;
use crate::records::position::Position;

#[repr(C)]
#[derive(Debug, Clone)]
pub struct CstTypeTypeof {
    pub base: CstNode,
    pub open_position: Position,
    pub close_position: Position,
}

impl crate::rtti::CstNodeClass for CstTypeTypeof {
    const CLASS_INDEX: i32 = crate::rtti::ast_rtti_index("CstTypeTypeof");
}

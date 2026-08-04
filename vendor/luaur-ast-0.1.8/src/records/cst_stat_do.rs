use crate::records::cst_node::CstNode;
use crate::records::position::Position;

#[repr(C)]
#[derive(Debug, Clone)]
pub struct CstStatDo {
    pub base: CstNode,
    pub stats_start_position: Position,
    pub end_position: Position,
}

impl crate::rtti::CstNodeClass for CstStatDo {
    const CLASS_INDEX: i32 = crate::rtti::ast_rtti_index("CstStatDo");
}

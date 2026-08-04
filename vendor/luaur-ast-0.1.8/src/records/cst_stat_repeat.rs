use crate::records::cst_node::CstNode;
use crate::records::position::Position;

#[repr(C)]
#[derive(Debug, Clone)]
pub struct CstStatRepeat {
    pub base: CstNode,
    pub until_position: Position,
}

impl crate::rtti::CstNodeClass for CstStatRepeat {
    const CLASS_INDEX: i32 = crate::rtti::ast_rtti_index("CstStatRepeat");
}

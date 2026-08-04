use crate::records::ast_array::AstArray;
use crate::records::cst_node::CstNode;
use crate::records::position::Position;

#[repr(C)]
#[derive(Debug, Clone)]
pub struct CstStatReturn {
    pub base: CstNode,
    pub comma_positions: AstArray<Position>,
}

impl crate::rtti::CstNodeClass for CstStatReturn {
    const CLASS_INDEX: i32 = crate::rtti::ast_rtti_index("CstStatReturn");
}

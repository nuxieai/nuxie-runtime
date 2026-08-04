use crate::records::ast_array::AstArray;
use crate::records::cst_node::CstNode;
use crate::records::position::Position;

#[repr(C)]
#[derive(Debug, Clone)]
pub struct CstTypeUnion {
    pub base: CstNode,
    pub leading_position: Position,
    pub separator_positions: AstArray<Position>,
}

impl crate::rtti::CstNodeClass for CstTypeUnion {
    const CLASS_INDEX: i32 = crate::rtti::cst_rtti_index("CstTypeUnion");
}

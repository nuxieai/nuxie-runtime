use crate::records::ast_array::AstArray;
use crate::records::cst_node::CstNode;
use crate::records::position::Position;

#[repr(C)]
#[derive(Debug, Clone)]
pub struct CstTypeIntersection {
    pub base: CstNode,
    pub leading_position: Position,
    pub separator_positions: AstArray<Position>,
}

impl crate::rtti::CstNodeClass for CstTypeIntersection {
    const CLASS_INDEX: i32 = crate::rtti::ast_rtti_index("CstTypeIntersection");
}

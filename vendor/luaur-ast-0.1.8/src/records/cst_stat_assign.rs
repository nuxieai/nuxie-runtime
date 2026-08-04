use crate::records::ast_array::AstArray;
use crate::records::cst_node::CstNode;
use crate::records::position::Position;

#[repr(C)]
#[derive(Debug, Clone)]
pub struct CstStatAssign {
    pub base: CstNode,
    pub vars_comma_positions: AstArray<Position>,
    pub equals_position: Position,
    pub values_comma_positions: AstArray<Position>,
}

impl crate::rtti::CstNodeClass for CstStatAssign {
    const CLASS_INDEX: i32 = crate::rtti::ast_rtti_index("CstStatAssign");
}

use crate::records::ast_array::AstArray;
use crate::records::cst_node::CstNode;
use crate::records::position::Position;

#[repr(C)]
#[derive(Debug, Clone)]
pub struct CstTypePackExplicit {
    pub base: CstNode,
    pub open_parentheses_position: Position,
    pub close_parentheses_position: Position,
    pub comma_positions: AstArray<Position>,
}

impl crate::rtti::CstNodeClass for CstTypePackExplicit {
    const CLASS_INDEX: i32 = crate::rtti::ast_rtti_index("CstTypePackExplicit");
}

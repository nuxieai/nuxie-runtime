use crate::records::ast_array::AstArray;
use crate::records::cst_node::CstNode;
use crate::records::position::Position;

#[repr(C)]
#[derive(Debug, Clone)]
pub struct CstParametrizedAttr {
    pub base: CstNode,
    pub open_paren_position: Position,
    pub close_paren_position: Position,
    pub args_comma_positions: AstArray<Position>,
}

impl crate::rtti::CstNodeClass for CstParametrizedAttr {
    const CLASS_INDEX: i32 = crate::rtti::ast_rtti_index("CstParametrizedAttr");
}

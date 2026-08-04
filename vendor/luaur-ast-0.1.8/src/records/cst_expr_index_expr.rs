use crate::records::cst_node::CstNode;
use crate::records::position::Position;

#[repr(C)]
#[derive(Debug, Clone)]
pub struct CstExprIndexExpr {
    pub base: CstNode,
    pub open_bracket_position: Position,
    pub close_bracket_position: Position,
}

impl crate::rtti::CstNodeClass for CstExprIndexExpr {
    const CLASS_INDEX: i32 = crate::rtti::ast_rtti_index("CstExprIndexExpr");
}

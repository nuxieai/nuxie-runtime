use crate::records::ast_array::AstArray;
use crate::records::cst_node::CstNode;
use crate::records::cst_type_instantiation::CstTypeInstantiation;
use crate::records::position::Position;

#[repr(C)]
#[derive(Debug, Clone)]
pub struct CstExprCall {
    pub base: CstNode,
    pub open_parens: Position,
    pub close_parens: Position,
    pub comma_positions: AstArray<Position>,
    pub explicit_types: *mut CstTypeInstantiation,
}

impl crate::rtti::CstNodeClass for CstExprCall {
    const CLASS_INDEX: i32 = crate::rtti::cst_rtti_index("CstExprCall");
}

use crate::records::cst_node::CstNode;
use crate::records::position::Position;

#[repr(C)]
#[derive(Debug, Clone)]
pub struct CstStatFunction {
    pub base: CstNode,
    pub function_keyword_position: Position,
}

impl crate::rtti::CstNodeClass for CstStatFunction {
    const CLASS_INDEX: i32 = crate::rtti::ast_rtti_index("CstStatFunction");
}

use crate::records::cst_node::CstNode;
use crate::records::position::Position;

#[repr(C)]
#[derive(Debug, Clone)]
pub struct CstGenericType {
    pub base: CstNode,
    pub default_equals_position: Position,
}

impl crate::rtti::CstNodeClass for CstGenericType {
    const CLASS_INDEX: i32 = crate::rtti::ast_rtti_index("CstGenericType");
}

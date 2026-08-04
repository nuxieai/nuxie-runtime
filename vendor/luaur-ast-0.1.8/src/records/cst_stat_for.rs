use crate::records::cst_node::CstNode;
use crate::records::position::Position;

#[repr(C)]
#[derive(Debug, Clone)]
pub struct CstStatFor {
    pub base: CstNode,
    pub annotation_colon_position: Position,
    pub equals_position: Position,
    pub end_comma_position: Position,
    pub step_comma_position: Position,
}

impl crate::rtti::CstNodeClass for CstStatFor {
    const CLASS_INDEX: i32 = crate::rtti::ast_rtti_index("CstStatFor");
}

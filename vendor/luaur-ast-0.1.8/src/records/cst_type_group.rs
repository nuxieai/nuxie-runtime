#[repr(C)]
#[derive(Debug, Clone)]
pub struct CstTypeGroup {
    pub base: crate::records::cst_node::CstNode,
    pub close_position: crate::records::position::Position,
}

impl crate::rtti::CstNodeClass for CstTypeGroup {
    const CLASS_INDEX: i32 = crate::rtti::ast_rtti_index("CstTypeGroup");
}

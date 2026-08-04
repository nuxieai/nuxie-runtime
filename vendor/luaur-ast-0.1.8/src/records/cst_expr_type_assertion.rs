#[repr(C)]
#[derive(Debug, Clone)]
pub struct CstExprTypeAssertion {
    pub base: crate::records::cst_node::CstNode,
    pub op_position: crate::records::position::Position,
}

impl crate::rtti::CstNodeClass for CstExprTypeAssertion {
    const CLASS_INDEX: i32 = crate::rtti::ast_rtti_index("CstExprTypeAssertion");
}

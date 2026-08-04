#[repr(C)]
#[derive(Debug, Clone)]
pub struct CstStatLocalFunction {
    pub base: crate::records::cst_node::CstNode,
    pub local_keyword_position: crate::records::position::Position,
    pub function_keyword_position: crate::records::position::Position,
}

impl crate::rtti::CstNodeClass for CstStatLocalFunction {
    const CLASS_INDEX: i32 = crate::rtti::ast_rtti_index("CstStatLocalFunction");
}

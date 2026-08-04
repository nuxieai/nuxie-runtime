#[repr(C)]
#[derive(Debug, Clone)]
pub struct CstStatTypeFunction {
    pub base: crate::records::cst_node::CstNode,
    pub type_keyword_position: crate::records::position::Position,
    pub function_keyword_position: crate::records::position::Position,
}

impl crate::rtti::CstNodeClass for CstStatTypeFunction {
    const CLASS_INDEX: i32 = crate::rtti::ast_rtti_index("CstStatTypeFunction");
}

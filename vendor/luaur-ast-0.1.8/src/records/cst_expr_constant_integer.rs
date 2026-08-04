#[repr(C)]
#[derive(Debug, Clone)]
pub struct CstExprConstantInteger {
    pub base: crate::records::cst_node::CstNode,
    pub value: crate::records::ast_array::AstArray<core::ffi::c_char>,
}

impl crate::rtti::CstNodeClass for CstExprConstantInteger {
    const CLASS_INDEX: i32 = crate::rtti::ast_rtti_index("CstExprConstantInteger");
}

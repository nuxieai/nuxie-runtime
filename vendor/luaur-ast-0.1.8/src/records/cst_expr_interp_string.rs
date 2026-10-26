#[repr(C)]
#[derive(Debug, Clone)]
pub struct CstExprInterpString {
    pub base: crate::records::cst_node::CstNode,
    pub source_strings:
        crate::records::ast_array::AstArray<crate::records::ast_array::AstArray<core::ffi::c_char>>,
    pub string_positions: crate::records::ast_array::AstArray<crate::records::position::Position>,
}

impl crate::rtti::CstNodeClass for CstExprInterpString {
    const CLASS_INDEX: i32 = crate::rtti::ast_rtti_index("CstExprInterpString");
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct CstTypeSingletonString {
    pub base: crate::records::cst_node::CstNode,
    pub source_string: crate::records::ast_array::AstArray<core::ffi::c_char>,
    pub quote_style: crate::enums::quote_style_cst::QuoteStyle,
    pub block_depth: u32,
}

impl crate::rtti::CstNodeClass for CstTypeSingletonString {
    const CLASS_INDEX: i32 = crate::rtti::ast_rtti_index("CstTypeSingletonString");
}

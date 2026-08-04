#[repr(C)]
#[derive(Debug, Clone)]
pub struct CstExprConstantString {
    pub base: crate::records::cst_node::CstNode,
    pub source_string: crate::records::ast_array::AstArray<core::ffi::c_char>,
    pub quote_style: crate::enums::quote_style_cst::QuoteStyle,
    pub block_depth: core::ffi::c_uint,
}

impl crate::rtti::CstNodeClass for CstExprConstantString {
    const CLASS_INDEX: i32 = crate::rtti::ast_rtti_index("CstExprConstantString");
}

#[allow(non_camel_case_types)]
pub type QuoteStyle = crate::enums::quote_style_cst::QuoteStyle;

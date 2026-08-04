use crate::records::ast_array::AstArray;
use crate::records::cst_expr_constant_string::CstExprConstantString;
use crate::records::cst_node::CstNode;
use crate::rtti::CstNodeClass;

impl CstExprConstantString {
    #[allow(non_snake_case)]
    pub fn new(
        source_string: AstArray<core::ffi::c_char>,
        quote_style: crate::enums::quote_style_cst::QuoteStyle,
        block_depth: core::ffi::c_uint,
    ) -> Self {
        luaur_common::LUAU_ASSERT!(
            block_depth == 0 || quote_style == crate::enums::quote_style_cst::QuoteStyle::QuotedRaw
        );

        Self {
            base: CstNode {
                class_index: <Self as CstNodeClass>::CLASS_INDEX,
            },
            source_string,
            quote_style,
            block_depth,
        }
    }
}

#[allow(non_snake_case)]
pub fn cst_expr_constant_string_cst_expr_constant_string(
    source_string: AstArray<core::ffi::c_char>,
    quote_style: crate::enums::quote_style_cst::QuoteStyle,
    block_depth: core::ffi::c_uint,
) -> CstExprConstantString {
    CstExprConstantString::new(source_string, quote_style, block_depth)
}

use crate::enums::quote_style_ast::QuoteStyle;
use crate::records::ast_array::AstArray;
use crate::records::ast_expr::AstExpr;
use crate::rtti::AstNodeClass;

#[repr(C)]
#[derive(Debug)]
pub struct AstExprConstantString {
    pub base: AstExpr,
    pub value: AstArray<core::ffi::c_char>,
    pub quote_style: QuoteStyle,
}

impl AstNodeClass for AstExprConstantString {
    const CLASS_INDEX: i32 = crate::rtti::ast_rtti_index("AstExprConstantString");
}

#[allow(non_upper_case_globals)]
impl AstExprConstantString {
    pub const QuotedSimple: QuoteStyle = QuoteStyle::QuotedSimple;
    pub const QuotedSingle: QuoteStyle = QuoteStyle::QuotedSingle;
    pub const QuotedRaw: QuoteStyle = QuoteStyle::QuotedRaw;
    pub const Unquoted: QuoteStyle = QuoteStyle::Unquoted;
}

#[allow(non_camel_case_types)]
pub type ast_expr_constant_string = AstExprConstantString;

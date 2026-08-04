#[repr(C)]
#[derive(Debug)]
#[allow(non_camel_case_types)]
pub struct AstExprConstantInteger {
    pub base: crate::records::ast_expr::AstExpr,
    pub value: i64,
    pub parse_result: crate::enums::constant_number_parse_result::ConstantNumberParseResult,
}

impl crate::rtti::AstNodeClass for AstExprConstantInteger {
    const CLASS_INDEX: i32 = crate::rtti::ast_rtti_index("AstExprConstantInteger");
}

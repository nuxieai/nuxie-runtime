use crate::enums::constant_number_parse_result::ConstantNumberParseResult;
use crate::records::ast_expr::AstExpr;
use crate::rtti::AstNodeClass;

#[repr(C)]
#[derive(Debug)]
pub struct AstExprConstantNumber {
    pub base: AstExpr,
    pub value: f64,
    pub parse_result: ConstantNumberParseResult,
}

impl AstNodeClass for AstExprConstantNumber {
    const CLASS_INDEX: i32 = crate::rtti::ast_rtti_index("AstExprConstantNumber");
}

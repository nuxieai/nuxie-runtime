use crate::records::ast_expr::AstExpr;
use crate::records::ast_expr_function::AstExprFunction;
use crate::records::ast_stat::AstStat;

#[repr(C)]
#[derive(Debug)]
pub struct AstStatFunction {
    pub base: AstStat,
    pub name: *mut AstExpr,
    pub func: *mut AstExprFunction,
}

impl crate::rtti::AstNodeClass for AstStatFunction {
    const CLASS_INDEX: i32 = crate::rtti::ast_rtti_index("AstStatFunction");
}

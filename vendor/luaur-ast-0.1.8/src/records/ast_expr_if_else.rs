use crate::records::ast_expr::AstExpr;

#[repr(C)]
#[derive(Debug)]
pub struct AstExprIfElse {
    pub base: AstExpr,
    pub condition: *mut AstExpr,
    pub has_then: bool,
    pub true_expr: *mut AstExpr,
    pub has_else: bool,
    pub false_expr: *mut AstExpr,
}

impl crate::rtti::AstNodeClass for AstExprIfElse {
    const CLASS_INDEX: i32 = crate::rtti::ast_rtti_index("AstExprIfElse");
}

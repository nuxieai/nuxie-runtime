use crate::records::ast_expr::AstExpr;
use crate::records::ast_type::AstType;

#[repr(C)]
#[derive(Debug)]
pub struct AstExprTypeAssertion {
    pub base: AstExpr,
    pub expr: *mut AstExpr,
    pub annotation: *mut AstType,
}

impl crate::rtti::AstNodeClass for AstExprTypeAssertion {
    const CLASS_INDEX: i32 = crate::rtti::ast_rtti_index("AstExprTypeAssertion");
}

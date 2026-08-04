use crate::records::ast_expr::AstExpr;

#[repr(C)]
#[derive(Debug)]
pub struct AstExprGroup {
    pub base: AstExpr,
    pub expr: *mut AstExpr,
}

impl crate::rtti::AstNodeClass for AstExprGroup {
    const CLASS_INDEX: i32 = crate::rtti::ast_rtti_index("AstExprGroup");
}

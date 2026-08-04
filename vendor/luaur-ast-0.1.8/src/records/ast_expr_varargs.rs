use crate::records::ast_expr::AstExpr;

#[repr(C)]
#[derive(Debug)]
#[allow(non_camel_case_types)]
pub struct AstExprVarargs {
    pub base: AstExpr,
}

impl crate::rtti::AstNodeClass for AstExprVarargs {
    const CLASS_INDEX: i32 = crate::rtti::ast_rtti_index("AstExprVarargs");
}

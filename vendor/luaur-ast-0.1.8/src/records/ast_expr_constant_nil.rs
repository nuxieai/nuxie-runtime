use crate::records::ast_expr::AstExpr;
use crate::rtti::AstNodeClass;

#[repr(C)]
#[derive(Debug)]
pub struct AstExprConstantNil {
    pub base: AstExpr,
}

impl AstNodeClass for AstExprConstantNil {
    const CLASS_INDEX: i32 = crate::rtti::ast_rtti_index("AstExprConstantNil");
}

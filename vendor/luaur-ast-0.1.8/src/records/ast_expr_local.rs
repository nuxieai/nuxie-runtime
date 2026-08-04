use crate::records::ast_expr::AstExpr;
use crate::records::ast_local::AstLocal;
use crate::rtti::AstNodeClass;

#[repr(C)]
#[derive(Debug, Clone)]
pub struct AstExprLocal {
    pub base: AstExpr,
    pub local: *mut AstLocal,
    pub upvalue: bool,
}

impl AstNodeClass for AstExprLocal {
    const CLASS_INDEX: i32 = crate::rtti::ast_rtti_index("AstExprLocal");
}

use crate::records::ast_expr::AstExpr;
use crate::records::ast_name::AstName;
use crate::rtti::AstNodeClass;

#[repr(C)]
#[derive(Debug)]
pub struct AstExprGlobal {
    pub base: AstExpr,
    pub name: AstName,
}

impl AstNodeClass for AstExprGlobal {
    const CLASS_INDEX: i32 = crate::rtti::ast_rtti_index("AstExprGlobal");
}

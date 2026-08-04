use crate::records::ast_expr::AstExpr;
use crate::records::ast_type::AstType;

#[repr(C)]
#[derive(Debug)]
pub struct AstTypeTypeof {
    pub base: AstType,
    pub expr: *mut AstExpr,
}

impl crate::rtti::AstNodeClass for AstTypeTypeof {
    const CLASS_INDEX: i32 = crate::rtti::ast_rtti_index("AstTypeTypeof");
}

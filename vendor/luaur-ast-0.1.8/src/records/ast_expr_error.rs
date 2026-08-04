use crate::records::ast_array::AstArray;
use crate::records::ast_expr::AstExpr;

#[repr(C)]
#[derive(Debug)]
pub struct AstExprError {
    pub base: AstExpr,
    pub expressions: AstArray<*mut AstExpr>,
    pub message_index: u32,
}

impl crate::rtti::AstNodeClass for AstExprError {
    const CLASS_INDEX: i32 = crate::rtti::ast_rtti_index("AstExprError");
}

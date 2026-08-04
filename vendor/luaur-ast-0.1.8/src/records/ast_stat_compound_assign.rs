use crate::records::ast_expr::AstExpr;
use crate::records::ast_expr_binary::AstExprBinary_Op;
use crate::records::ast_stat::AstStat;

#[repr(C)]
#[derive(Debug)]
pub struct AstStatCompoundAssign {
    pub base: AstStat,
    pub op: AstExprBinary_Op,
    pub var: *mut AstExpr,
    pub value: *mut AstExpr,
}

impl crate::rtti::AstNodeClass for AstStatCompoundAssign {
    const CLASS_INDEX: i32 = crate::rtti::ast_rtti_index("AstStatCompoundAssign");
}

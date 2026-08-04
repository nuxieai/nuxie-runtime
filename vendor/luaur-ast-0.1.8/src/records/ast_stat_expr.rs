use crate::records::ast_expr::AstExpr;
use crate::records::ast_stat::AstStat;

#[repr(C)]
#[derive(Debug)]
pub struct AstStatExpr {
    pub base: AstStat,
    pub expr: *mut AstExpr,
}

impl crate::rtti::AstNodeClass for AstStatExpr {
    const CLASS_INDEX: i32 = crate::rtti::ast_rtti_index("AstStatExpr");
}

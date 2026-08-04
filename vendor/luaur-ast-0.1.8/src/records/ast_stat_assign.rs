#[repr(C)]
#[derive(Debug, Clone)]
pub struct AstStatAssign {
    pub base: crate::records::ast_stat::AstStat,
    pub vars: crate::records::ast_array::AstArray<*mut crate::records::ast_expr::AstExpr>,
    pub values: crate::records::ast_array::AstArray<*mut crate::records::ast_expr::AstExpr>,
}

impl crate::rtti::AstNodeClass for AstStatAssign {
    const CLASS_INDEX: i32 = crate::rtti::ast_rtti_index("AstStatAssign");
}

use crate::records::ast_array::AstArray;
use crate::records::ast_expr::AstExpr;
use crate::records::ast_stat::AstStat;

#[repr(C)]
#[derive(Debug)]
pub struct AstStatError {
    pub base: AstStat,
    pub expressions: AstArray<*mut AstExpr>,
    pub statements: AstArray<*mut AstStat>,
    pub message_index: u32,
}

impl crate::rtti::AstNodeClass for AstStatError {
    const CLASS_INDEX: i32 = crate::rtti::ast_rtti_index("AstStatError");
}

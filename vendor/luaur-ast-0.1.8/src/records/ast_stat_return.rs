use crate::records::ast_array::AstArray;
use crate::records::ast_expr::AstExpr;
use crate::records::ast_stat::AstStat;

#[repr(C)]
#[derive(Debug, Clone)]
#[allow(non_camel_case_types)]
pub struct AstStatReturn {
    pub base: AstStat,
    pub list: AstArray<*mut AstExpr>,
}

impl crate::rtti::AstNodeClass for AstStatReturn {
    const CLASS_INDEX: i32 = crate::rtti::ast_rtti_index("AstStatReturn");
}

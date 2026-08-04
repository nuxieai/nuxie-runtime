use crate::records::ast_array::AstArray;
use crate::records::ast_expr::AstExpr;
use crate::records::ast_local::AstLocal;
use crate::records::ast_stat::AstStat;
use crate::records::location::Location;

#[repr(C)]
#[derive(Debug, Clone)]
pub struct AstStatLocal {
    pub base: AstStat,
    pub vars: AstArray<*mut AstLocal>,
    pub values: AstArray<*mut AstExpr>,
    pub is_const: bool,
    pub is_exported: bool,
    pub equals_sign_location: Option<Location>,
}

impl crate::rtti::AstNodeClass for AstStatLocal {
    const CLASS_INDEX: i32 = crate::rtti::ast_rtti_index("AstStatLocal");
}

#[allow(non_upper_case_globals)]
pub const AstStatLocal_ClassIndex: i32 = <AstStatLocal as crate::rtti::AstNodeClass>::CLASS_INDEX;

use crate::records::ast_expr_function::AstExprFunction;
use crate::records::ast_name::AstName;
use crate::records::ast_stat::AstStat;
use crate::records::location::Location;

#[repr(C)]
#[derive(Debug, Clone)]
pub struct AstStatTypeFunction {
    pub base: AstStat,
    pub name: AstName,
    pub name_location: Location,
    pub body: *mut AstExprFunction,
    pub exported: bool,
    pub has_errors: bool,
}

impl crate::rtti::AstNodeClass for AstStatTypeFunction {
    const CLASS_INDEX: i32 = crate::rtti::ast_rtti_index("AstStatTypeFunction");
}

use crate::records::ast_expr::AstExpr;
use crate::records::ast_local::AstLocal;
use crate::records::ast_stat::AstStat;
use crate::records::ast_stat_block::AstStatBlock;
use crate::records::location::Location;

#[repr(C)]
#[derive(Debug)]
pub struct AstStatFor {
    pub base: AstStat,
    pub var: *mut AstLocal,
    pub from: *mut AstExpr,
    pub to: *mut AstExpr,
    pub step: *mut AstExpr,
    pub body: *mut AstStatBlock,
    pub has_do: bool,
    pub do_location: Location,
}

impl crate::rtti::AstNodeClass for AstStatFor {
    const CLASS_INDEX: i32 = crate::rtti::ast_rtti_index("AstStatFor");
}

#[allow(non_upper_case_globals)]
pub const AstStatFor_ClassIndex: i32 = <AstStatFor as crate::rtti::AstNodeClass>::CLASS_INDEX;

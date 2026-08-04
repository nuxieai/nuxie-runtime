use crate::records::ast_array::AstArray;
use crate::records::ast_expr::AstExpr;
use crate::records::ast_local::AstLocal;
use crate::records::ast_stat::AstStat;
use crate::records::ast_stat_block::AstStatBlock;
use crate::records::location::Location;

#[repr(C)]
#[derive(Debug)]
pub struct AstStatForIn {
    pub base: AstStat,
    pub vars: AstArray<*mut AstLocal>,
    pub values: AstArray<*mut AstExpr>,
    pub body: *mut AstStatBlock,
    pub has_in: bool,
    pub in_location: Location,
    pub has_do: bool,
    pub do_location: Location,
}

impl crate::rtti::AstNodeClass for AstStatForIn {
    const CLASS_INDEX: i32 = crate::rtti::ast_rtti_index("AstStatForIn");
}

#[allow(non_upper_case_globals)]
pub const AstStatForIn_ClassIndex: i32 = <AstStatForIn as crate::rtti::AstNodeClass>::CLASS_INDEX;

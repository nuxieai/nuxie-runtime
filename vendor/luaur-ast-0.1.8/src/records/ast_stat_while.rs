use crate::records::ast_expr::AstExpr;
use crate::records::ast_stat::AstStat;
use crate::records::ast_stat_block::AstStatBlock;
use crate::records::location::Location;

#[repr(C)]
#[derive(Debug)]
pub struct AstStatWhile {
    pub base: AstStat,
    pub condition: *mut AstExpr,
    pub body: *mut AstStatBlock,
    pub has_do: bool,
    pub do_location: Location,
}

impl crate::rtti::AstNodeClass for AstStatWhile {
    const CLASS_INDEX: i32 = crate::rtti::ast_rtti_index("AstStatWhile");
}

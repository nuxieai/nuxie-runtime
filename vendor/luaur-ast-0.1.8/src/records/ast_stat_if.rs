use crate::records::ast_expr::AstExpr;
use crate::records::ast_stat::AstStat;
use crate::records::ast_stat_block::AstStatBlock;
use crate::records::location::Location;

#[repr(C)]
#[derive(Debug)]
pub struct AstStatIf {
    pub base: AstStat,
    pub condition: *mut AstExpr,
    pub thenbody: *mut AstStatBlock,
    pub elsebody: *mut AstStat,
    pub then_location: Option<Location>,
    pub else_location: Option<Location>,
}

impl crate::rtti::AstNodeClass for AstStatIf {
    const CLASS_INDEX: i32 = crate::rtti::ast_rtti_index("AstStatIf");
}

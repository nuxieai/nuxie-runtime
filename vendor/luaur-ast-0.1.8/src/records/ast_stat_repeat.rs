use crate::records::ast_expr::AstExpr;
use crate::records::ast_stat::AstStat;
use crate::records::ast_stat_block::AstStatBlock;

#[repr(C)]
#[derive(Debug)]
pub struct AstStatRepeat {
    pub base: AstStat,
    pub condition: *mut AstExpr,
    pub body: *mut AstStatBlock,
    pub DEPRECATED_hasUntil: bool,
}

impl crate::rtti::AstNodeClass for AstStatRepeat {
    const CLASS_INDEX: i32 = crate::rtti::ast_rtti_index("AstStatRepeat");
}

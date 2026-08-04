use crate::records::ast_stat::AstStat;
use crate::rtti::AstNodeClass;

#[repr(C)]
#[derive(Debug)]
pub struct AstStatBreak {
    pub base: AstStat,
}

impl AstNodeClass for AstStatBreak {
    const CLASS_INDEX: i32 = crate::rtti::ast_rtti_index("AstStatBreak");
}

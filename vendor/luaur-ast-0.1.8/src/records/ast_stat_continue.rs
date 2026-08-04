use crate::records::ast_stat::AstStat;

#[repr(C)]
#[derive(Debug)]
pub struct AstStatContinue {
    pub base: AstStat,
}

impl crate::rtti::AstNodeClass for AstStatContinue {
    const CLASS_INDEX: i32 = crate::rtti::ast_rtti_index("AstStatContinue");
}

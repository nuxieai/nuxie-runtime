use crate::records::ast_array::AstArray;
use crate::records::ast_stat::AstStat;
use crate::records::location::Location;

#[repr(C)]
#[derive(Debug)]
pub struct AstStatBlock {
    pub base: AstStat,
    pub body: AstArray<*mut AstStat>,
    pub has_end: bool,
}

impl crate::rtti::AstNodeClass for AstStatBlock {
    const CLASS_INDEX: i32 = crate::rtti::ast_rtti_index("AstStatBlock");
}

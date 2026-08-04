use crate::records::ast_array::AstArray;
use crate::records::ast_local::AstLocal;
use crate::records::ast_stat::AstStat;
use crate::type_aliases::ast_class_member::AstClassMember;

#[repr(C)]
#[derive(Debug, Clone)]
pub struct AstStatClass {
    pub base: AstStat,
    pub name: *mut AstLocal,
    pub members: AstArray<AstClassMember>,
    pub exported: bool,
}

impl crate::rtti::AstNodeClass for AstStatClass {
    const CLASS_INDEX: i32 = crate::rtti::ast_rtti_index("AstStatClass");
}

use crate::records::ast_type_list::AstTypeList;
use crate::records::ast_type_pack::AstTypePack;

#[repr(C)]
#[derive(Debug)]
pub struct AstTypePackExplicit {
    pub base: AstTypePack,
    pub type_list: AstTypeList,
}

impl crate::rtti::AstNodeClass for AstTypePackExplicit {
    const CLASS_INDEX: i32 = crate::rtti::ast_rtti_index("AstTypePackExplicit");
}

use crate::records::ast_name::AstName;
use crate::records::ast_node::AstNode;
use crate::records::ast_type_pack::AstTypePack;

#[repr(C)]
#[derive(Debug)]
pub struct AstGenericTypePack {
    pub base: AstNode,
    pub name: AstName,
    pub default_value: *mut AstTypePack,
}

impl crate::rtti::AstNodeClass for AstGenericTypePack {
    const CLASS_INDEX: i32 = crate::rtti::ast_rtti_index("AstGenericTypePack");
}

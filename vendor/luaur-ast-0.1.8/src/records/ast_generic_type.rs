use crate::records::ast_name::AstName;
use crate::records::ast_node::AstNode;
use crate::records::ast_type::AstType;

#[repr(C)]
#[derive(Debug, Clone)]
pub struct AstGenericType {
    pub base: AstNode,
    pub name: AstName,
    pub default_value: *mut AstType,
}

impl crate::rtti::AstNodeClass for AstGenericType {
    const CLASS_INDEX: i32 = crate::rtti::ast_rtti_index("AstGenericType");
}

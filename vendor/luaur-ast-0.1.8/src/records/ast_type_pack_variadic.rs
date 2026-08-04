use crate::records::ast_type::AstType;
use crate::records::ast_type_pack::AstTypePack;

#[repr(C)]
#[derive(Debug)]
pub struct AstTypePackVariadic {
    pub base: AstTypePack,
    pub variadic_type: *mut AstType,
}

impl crate::rtti::AstNodeClass for AstTypePackVariadic {
    const CLASS_INDEX: i32 = crate::rtti::ast_rtti_index("AstTypePackVariadic");
}

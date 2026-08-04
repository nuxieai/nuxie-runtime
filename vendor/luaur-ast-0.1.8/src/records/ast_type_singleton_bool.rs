#[repr(C)]
#[derive(Debug, Clone)]
pub struct AstTypeSingletonBool {
    pub base: crate::records::ast_type::AstType,
    pub value: bool,
}

impl crate::rtti::AstNodeClass for AstTypeSingletonBool {
    const CLASS_INDEX: i32 = crate::rtti::ast_rtti_index("AstTypeSingletonBool");
}

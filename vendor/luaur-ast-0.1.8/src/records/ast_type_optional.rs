use crate::records::ast_type::AstType;

#[repr(C)]
#[derive(Debug)]
pub struct AstTypeOptional {
    pub base: AstType,
    pub type_: *mut AstType,
}

impl crate::rtti::AstNodeClass for AstTypeOptional {
    const CLASS_INDEX: i32 = crate::rtti::ast_rtti_index("AstTypeOptional");
}
